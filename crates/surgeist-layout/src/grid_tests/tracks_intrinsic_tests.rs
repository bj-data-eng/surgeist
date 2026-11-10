use super::fixtures::{
    Fri08C02StretchTreeInput, Fri08C02TrackAxis, assert_fri08_c02_fit_content_flex_composes,
    assert_fri08_c02_stretch_intrinsic_minimums,
    assert_fri08_c03_containing_block_percentage_controls, compute_oracle_grid,
    compute_oracle_grid_output, computed_overflow, fri04_c03_grid_track_nested,
    fri04_c03_grid_track_percentage_nested, fri04_c03_grid_track_value,
    fri04_c04_grid_dispatch_input, fri05_c05_grid_sizing_input,
    fri06_mr02_geometry_error_largest_finite, fri08_c01_placement_output,
    fri08_c02_auto_fit_output, fri08_c02_auto_fit_repeat, fri08_c02_fit_content_track,
    fri08_c02_flex_track, fri08_c02_stretch_track, fri08_c02_stretch_tree,
    fri08_c02_track_mix_tree, fri08_c02_track_sizes, fri08_c03_auto_fit_batch,
    fri08_c03_auto_fit_named_repeat, fri08_c03_intrinsic_facts, fri08_c03_intrinsic_projected_item,
    fri08_c04_standalone_intrinsic_minimum_width, inherited_placement_member, invalid_numeric_lp,
    lp, subgrid_track_of, track_component_flex,
};
use super::*;
use crate::test_support::readback::{self, Phase};

// Numeric fixtures author item incidence directly. The argument is a zero-based
// single-track index; the contribution row carries the actual1-based grid lines.
fn single_track_item<S: LayoutScalar>(
    track: usize,
    minimum: S,
    min_content: S,
    max_content: S,
) -> GridIntrinsicContributionRow<S> {
    GridIntrinsicContributionRow {
        span: GridTrackSpan::new(track + 1, track + 2),
        contributions: GridTrackContributions {
            minimum,
            min_content,
            max_content,
        },
    }
}

fn grid_item_sizing_with_grid_flow_status<S: LayoutScalar>(
    child_style: &NodeInputOf<S>,
    container_style: &NodeInputOf<S>,
    area_size: Size<S>,
    containing_physical_size: Size<Option<S>>,
    grid_flow_axes: crate::geometry::FlowAxes,
) -> Result<GridItemSizing<S>, SizingResolutionError<S>> {
    super::child::grid_item_sizing_with_grid_flow_status(
        &grid_item_projection!(child_style),
        &grid_container_projection!(container_style),
        Size::new(
            child_style
                .grid_template_columns
                .iter()
                .any(|component| matches!(component, TrackComponentOf::Subgrid(_))),
            child_style
                .grid_template_rows
                .iter()
                .any(|component| matches!(component, TrackComponentOf::Subgrid(_))),
        ),
        area_size,
        containing_physical_size,
        grid_flow_axes,
    )
}

#[test]
fn fri08_c03_containing_block_percentage_controls_f32() {
    assert_fri08_c03_containing_block_percentage_controls::<f32>();
}

#[test]
fn fri08_c03_containing_block_percentage_controls_f64() {
    assert_fri08_c03_containing_block_percentage_controls::<f64>();
}

#[test]
fn fri08_c03_auto_fit_explicit_overlap_retains_one_track_and_running_offsets() {
    let tree = PublicLayoutTreeOf::<f32>::new().children(1, [2, 3]).style(
        1,
        NodeInput {
            display: Display::GridLanes,
            size: Size::new(PreferredSize::px(190.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![fri08_c03_auto_fit_named_repeat(TrackRepeat::AutoFit)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            gap: Size::new(Length::px(10.0), Length::px(5.0)),
            justify_content: Some(AlignContent::Center),
            align_content: Some(AlignContent::Start),
            ..NodeInput::DEFAULT
        },
    );
    let named_first = RawGridPlacement::new(
        RawGridLine::NamedLine {
            name: "slot".to_string(),
            index: 1,
        },
        RawGridLine::Auto,
    );
    let tree = [2, 3].into_iter().fold(tree, |tree, node| {
        tree.style(
            node,
            NodeInput {
                raw_grid_column: named_first.clone(),
                size: Size::new(PreferredSize::AUTO, PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
    });
    let batch = fri08_c03_auto_fit_batch(&tree, Size::new(190.0, 40.0));
    let first = fri08_c01_placement_output(&batch, 2);
    let second = fri08_c01_placement_output(&batch, 3);

    assert_eq!((first.location.x, first.size.width), (75.0, 40.0));
    assert_eq!((second.location.x, second.size.width), (75.0, 40.0));
    assert_eq!((first.location.y, second.location.y), (0.0, 15.0));
}

#[test]
fn fri08_c03_auto_fit_zero_automatic_demand_collapses_all_unused_tracks_and_gutters() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c03_auto_fit_named_repeat(TrackRepeat::AutoFit)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_lines(1, 4).expect("all retained line identities"),
                grid_row: GridPlacement::try_lines(1, 2).expect("single row"),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );
    let batch = fri08_c03_auto_fit_batch(&tree, Size::new(140.0, 20.0));
    let absolute = fri08_c01_placement_output(&batch, 2);

    assert_eq!((absolute.location.x, absolute.size.width), (70.0, 0.0));

    let rows_tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(140.0)),
                grid_template_columns: vec![TrackComponent::px(20.0)],
                grid_template_rows: vec![fri08_c03_auto_fit_named_repeat(TrackRepeat::AutoFit)],
                grid_lanes_flow: crate::GridLanesFlow::new(crate::GridAxisKind::Row, false, false),
                gap: Size::new(Length::ZERO, Length::px(10.0)),
                align_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_lines(1, 2).expect("single column"),
                grid_row: GridPlacement::try_lines(1, 4).expect("all retained line identities"),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );
    let rows_batch = fri08_c03_auto_fit_batch(&rows_tree, Size::new(20.0, 140.0));
    let rows_absolute = fri08_c01_placement_output(&rows_batch, 2);
    assert_eq!(
        (rows_absolute.location.y, rows_absolute.size.height),
        (70.0, 0.0)
    );
}

#[test]
fn fri08_c04_standalone_intrinsic_minimum_preserves_min_and_max_content_phases() {
    for (minimum, expected) in [
        (MinSizeOf::<f32>::MIN_CONTENT, 20.0),
        (MinSizeOf::<f32>::MAX_CONTENT, 80.0),
    ] {
        assert_eq!(
            fri08_c04_standalone_intrinsic_minimum_width(minimum.clone()),
            expected,
            "{minimum:?} must retain its standalone measurement phase"
        );
    }
}

fn assert_fri08_c04_standalone_edges_gaps_and_percentage<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3, 4])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGrid,
                size: Size::new(
                    PreferredSizeOf::px(scalar(200.0)),
                    PreferredSizeOf::px(scalar(110.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                gap: Size::new(LengthOf::ZERO, LengthOf::px(scalar(10.0))),
                justify_content: Some(AlignContent::Stretch),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: subgrid_track_of(),
                grid_column: GridPlacement::try_line(1).expect("standalone parent column"),
                grid_row: GridPlacement::try_line_span(1, 2).expect("inherited row span"),
                gap: Size::new(LengthOf::ZERO, LengthOf::px(scalar(20.0))),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(13.0)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(scalar(11.0)),
                ),
                border: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::px(scalar(5.0)),
                    LengthOf::ZERO,
                    LengthOf::px(scalar(3.0)),
                ),
                padding: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::px(scalar(9.0)),
                    LengthOf::ZERO,
                    LengthOf::px(scalar(7.0)),
                ),
                justify_content: Some(AlignContent::Stretch),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::percent(scalar(0.5)),
                    PreferredSizeOf::px(scalar(50.0)),
                ),
                grid_column: GridPlacement::try_line(1).expect("local percentage column"),
                grid_row: GridPlacement::try_line(1).expect("first inherited row"),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::percent(scalar(0.5)),
                    PreferredSizeOf::px(scalar(50.0)),
                ),
                grid_column: GridPlacement::try_line(1).expect("local percentage column"),
                grid_row: GridPlacement::try_line(2).expect("second inherited row"),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(&tree, 1, {
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
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("standalone percentage and unequal-gap layout succeeds");
    let wrapper = fri08_c01_placement_output(&batch, 2);
    let first = fri08_c01_placement_output(&batch, 3);
    let second = fri08_c01_placement_output(&batch, 4);
    assert_eq!(wrapper.size.width, scalar(176.0));
    assert_eq!(first.size.width, scalar(76.0));
    assert_eq!(second.size.width, scalar(76.0));
    assert_eq!(second.location.y - first.location.y, scalar(70.0));
}

#[test]
fn fri08_c04_standalone_context_applies_percentage_unequal_gaps_and_mbp_once() {
    assert_fri08_c04_standalone_edges_gaps_and_percentage::<f32>();
    assert_fri08_c04_standalone_edges_gaps_and_percentage::<f64>();
}

#[test]
fn fri08_c03_intrinsic_definite_exact_and_automatic_spans_project_to_every_active_start() {
    let input = LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: None,
        gap: 0.0,
        tracks: vec![TrackSizing::AUTO; 4],
        content_sized_tracks: vec![0, 1, 2, 3],
        items: vec![
            LaneIntrinsicItem::definite(
                "definite",
                LaneTrackSpan::new(2, 4),
                fri08_c03_intrinsic_facts(20.0, 30.0, 40.0),
            )
            .expect("definite span fits"),
            LaneIntrinsicItem::indefinite(
                "auto-one",
                LaneTrackSpanLength::new(1).expect("one is nonzero"),
                fri08_c03_intrinsic_facts(10.0, 20.0, 30.0),
            ),
            LaneIntrinsicItem::indefinite(
                "auto-two",
                LaneTrackSpanLength::new(2).expect("two is nonzero"),
                fri08_c03_intrinsic_facts(10.0, 20.0, 30.0),
            ),
        ],
    };
    let report = lane_intrinsic_sizing(input)
        .expect("candidate projection has finite values")
        .expect("candidate projection has valid spans");

    assert_eq!(report.definite_items[0].span, LaneTrackSpan::new(2, 4));
    assert_eq!(
        report
            .converted_indefinite_items
            .iter()
            .map(|item| item.span)
            .collect::<Vec<_>>(),
        vec![
            LaneTrackSpan::new(1, 2),
            LaneTrackSpan::new(2, 3),
            LaneTrackSpan::new(3, 4),
            LaneTrackSpan::new(4, 5),
            LaneTrackSpan::new(1, 3),
            LaneTrackSpan::new(2, 4),
            LaneTrackSpan::new(3, 5),
        ]
    );
}

fn assert_fri08_c03_intrinsic_fixed_content_gap_distribution<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let input = LaneIntrinsicSizingInputOf::<S> {
        axis: GridAxisKind::Column,
        available: None,
        gap: scalar(10.0),
        tracks: vec![
            TrackSizingOf::AUTO,
            TrackSizingOf::px(scalar(20.0)),
            TrackSizingOf::AUTO,
        ],
        content_sized_tracks: vec![0, 2],
        items: vec![LaneIntrinsicItemOf::indefinite(
            "mixed-span",
            LaneTrackSpanLength::new(3).expect("three is nonzero"),
            fri08_c03_intrinsic_facts(100.0, 120.0, 160.0),
        )],
    };
    let report = lane_intrinsic_sizing(input)
        .expect("mixed projection values are finite")
        .expect("mixed projection span is valid");
    assert_eq!(
        report.final_track_sizes,
        vec![scalar(30.0), scalar(20.0), scalar(30.0)]
    );
}

#[test]
fn fri08_c03_intrinsic_fixed_content_gap_distribution_is_scalar_deterministic() {
    assert_fri08_c03_intrinsic_fixed_content_gap_distribution::<f32>();
    assert_fri08_c03_intrinsic_fixed_content_gap_distribution::<f64>();
}

#[test]
fn fri08_c03_intrinsic_collapsed_auto_fit_tracks_are_not_candidate_starts() {
    let input = LaneIntrinsicSizingInput {
        axis: GridAxisKind::Column,
        available: None,
        gap: 10.0,
        tracks: vec![TrackSizing::AUTO, TrackSizing::px(0.0), TrackSizing::AUTO],
        content_sized_tracks: vec![0, 2],
        items: Vec::new(),
    };
    let items = vec![fri08_c03_intrinsic_projected_item(
        "active-only",
        1,
        None,
        LaneIntrinsicEdgeFactsOf::default(),
        fri08_c03_intrinsic_facts(20.0, 30.0, 40.0),
    )];
    let gutters = OrdinaryGridAxisGuttersOf::new(3, &[false, true, false], 10.0);
    let report = lane_intrinsic_sizing_projected_with::<(), _, core::convert::Infallible>(
        &input,
        &items,
        Some(&gutters),
        LayoutErrorSite::Standalone,
    )
    .expect("active candidate projection has finite values")
    .expect("active candidate projection has valid spans");
    assert_eq!(
        report
            .converted_indefinite_items
            .iter()
            .map(|item| item.span)
            .collect::<Vec<_>>(),
        vec![LaneTrackSpan::new(1, 2), LaneTrackSpan::new(3, 4)]
    );
    assert_eq!(report.final_track_sizes, [20.0, 0.0, 20.0]);
}

#[test]
fn fri08_c02_stretch_minmax_zero_auto_uses_definite_remaining_space() {
    let (tree, flow_axes, viewport) = fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
        display: Display::Grid,
        axis: Fri08C02TrackAxis::Columns,
        writing_mode: WritingMode::HorizontalTb,
        definite_axis_size: Some(100.0),
        viewport_axis_size: 100.0,
        gap: 0.0,
        alignment: Some(AlignContent::Stretch),
        tracks: vec![fri08_c02_stretch_track(MinTrackSizingOf::px(0.0))],
        measurements: &[0.0],
    });

    assert_eq!(
        fri08_c02_track_sizes(&tree, flow_axes, viewport, Fri08C02TrackAxis::Columns, 1,),
        [100.0]
    );
}

#[test]
fn fri08_c02_stretch_intrinsic_minimums_match_both_axes_writing_modes_and_scalars() {
    assert_fri08_c02_stretch_intrinsic_minimums::<f32>();
    assert_fri08_c02_stretch_intrinsic_minimums::<f64>();
}

#[test]
fn fri08_c02_stretch_excludes_other_maxima_after_fit_and_flex_use() {
    let tracks = vec![
        fri08_c02_stretch_track(MinTrackSizingOf::px(0.0)),
        TrackComponentOf::px(10.0),
        fri08_c02_fit_content_track(30.0, 0.0),
        TrackComponentOf::minmax(MinTrackSizingOf::px(0.0), MaxTrackSizingOf::MIN_CONTENT),
        TrackComponentOf::minmax(MinTrackSizingOf::px(0.0), MaxTrackSizingOf::MAX_CONTENT),
        fri08_c02_flex_track(0.5),
    ];
    let (tree, flow_axes, viewport) = fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
        display: Display::Grid,
        axis: Fri08C02TrackAxis::Columns,
        writing_mode: WritingMode::HorizontalTb,
        definite_axis_size: Some(300.0),
        viewport_axis_size: 300.0,
        gap: 0.0,
        alignment: Some(AlignContent::Stretch),
        tracks,
        measurements: &[0.0, 0.0, 20.0, 15.0, 25.0, 0.0],
    });

    assert_eq!(
        fri08_c02_track_sizes(&tree, flow_axes, viewport, Fri08C02TrackAxis::Columns, 6,),
        [115.0, 10.0, 20.0, 15.0, 25.0, 115.0]
    );
}

#[test]
fn fri08_c02_stretch_uses_only_active_gaps_and_noncollapsed_auto_fit_tracks() {
    let repeat = TrackComponentOf::Repeat(
        TrackRepetitionOf::auto_fit_components(vec![fri08_c02_stretch_track(
            MinTrackSizingOf::px(40.0),
        )])
        .expect("valid fixed-minimum auto-fit repetition"),
    );
    let (tree, _, viewport) = fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
        display: Display::Grid,
        axis: Fri08C02TrackAxis::Columns,
        writing_mode: WritingMode::HorizontalTb,
        definite_axis_size: Some(140.0),
        viewport_axis_size: 140.0,
        gap: 10.0,
        alignment: Some(AlignContent::Stretch),
        tracks: vec![repeat],
        measurements: &[0.0, 0.0],
    });
    let batch = compute_layout(&tree, 1, {
        let axes = viewport.map(|value| {
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(value).unwrap()),
                SizingOperation::Normal,
            )
        });
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
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("valid collapsed stretch grid");
    let first = fri08_c01_placement_output(&batch, 2);
    let second = fri08_c01_placement_output(&batch, 3);

    assert_eq!((first.location.x, first.size.width), (0.0, 65.0));
    assert_eq!((second.location.x, second.size.width), (75.0, 65.0));
}

#[test]
fn fri08_c02_stretch_requires_normal_or_stretch_positive_definite_remainder() {
    let resolve = |definite_axis_size, alignment| {
        let (tree, flow_axes, viewport) = fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
            display: Display::Grid,
            axis: Fri08C02TrackAxis::Columns,
            writing_mode: WritingMode::HorizontalTb,
            definite_axis_size,
            viewport_axis_size: 100.0,
            gap: 0.0,
            alignment,
            tracks: vec![fri08_c02_stretch_track(
                MinTrackSizingOf::<f32>::MIN_CONTENT,
            )],
            measurements: &[20.0],
        });
        fri08_c02_track_sizes(&tree, flow_axes, viewport, Fri08C02TrackAxis::Columns, 1)[0]
    };

    assert_eq!(resolve(Some(100.0), None), 100.0);
    assert_eq!(resolve(Some(100.0), Some(AlignContent::Stretch)), 100.0);
    assert_eq!(resolve(Some(100.0), Some(AlignContent::Start)), 20.0);
    assert_eq!(resolve(Some(100.0), Some(AlignContent::Center)), 20.0);
    assert_eq!(
        {
            // Each listed row is one authored single-track item, including zero-sized items.
            let tracks = &[TrackSizingOf::minmax(
                MinTrackSizingOf::MIN_CONTENT,
                MaxTrackSizingOf::AUTO,
            )];
            let items = [single_track_item(0, 20.0, 20.0, 20.0)];
            let phases = resolve_grid_contribution_rows(
                SpanningTrackContext {
                    tracks,
                    basis: None,
                    available: GridTrackSizingConstraint {
                        operation: SizingOperation::MaxContent,
                        available_extent: None,
                    },
                    gap: 0.0,
                    gutters: None,
                    base_floors: &[],
                },
                &items,
            )
            .expect("authored item incidence resolves checked phases");
            resolve_tracks_from_phases(ResolvedAxisTrackInput {
                tracks,
                phases: &phases,
                items: &items,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                flex_bounds: GridTrackFinishingBounds::NONE,
                stretch_size: None,
                gap: 0.0,
                alignment: AlignContent::Stretch,
                gutters: None,
            })
            .expect("authored track phases finish without losing item spans")
        },
        [20.0]
    );
    assert_eq!(resolve(Some(10.0), Some(AlignContent::Stretch)), 20.0);
}

#[test]
fn fri08_c02_fit_content_columns_continue_into_flexible_expansion() {
    assert_fri08_c02_fit_content_flex_composes::<f32>(
        Fri08C02TrackAxis::Columns,
        WritingMode::HorizontalTb,
    );
}

#[test]
fn fri08_c02_fit_content_rows_continue_into_flexible_expansion() {
    assert_fri08_c02_fit_content_flex_composes::<f32>(
        Fri08C02TrackAxis::Rows,
        WritingMode::HorizontalTb,
    );
}

#[test]
fn fri08_c02_fit_content_percentage_intrinsic_companions_and_sub_one_flex_retain_semantics() {
    for definite_axis_size in [Some(200.0), None] {
        let (tree, flow_axes, viewport) = fri08_c02_track_mix_tree::<f32>(
            Display::Grid,
            Fri08C02TrackAxis::Columns,
            WritingMode::HorizontalTb,
            (0.0, 0.25),
            definite_axis_size,
            vec![
                TrackComponentOf::MIN_CONTENT,
                TrackComponentOf::MAX_CONTENT,
                fri08_c02_flex_track::<f32>(0.25),
                fri08_c02_flex_track::<f32>(0.25),
            ],
            &[20.0, 10.0, 30.0, 8.0, 12.0],
        );
        let sizes =
            fri08_c02_track_sizes(&tree, flow_axes, viewport, Fri08C02TrackAxis::Columns, 5);
        let expected = if definite_axis_size.is_some() {
            [20.0, 10.0, 30.0, 35.0, 35.0]
        } else {
            [20.0, 10.0, 30.0, 8.0, 12.0]
        };
        for (actual, expected) in sizes.into_iter().zip(expected) {
            assert!((actual - expected).abs() <= 0.001);
        }
    }
}

#[test]
fn fri08_c02_fit_content_spanning_contribution_caps_fit_and_grows_companion() {
    let scalar = f32::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3, 4])
        .children(4, [5, 6])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![
                    TrackComponent::MAX_CONTENT,
                    fri08_c02_fit_content_track(10.0, 0.0),
                ],
                grid_template_rows: vec![TrackComponent::px(40.0)],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("first contribution track"),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(2).expect("fit-content contribution track"),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
                grid_column: GridPlacement::try_line_span(1, 2).expect("two-track contribution"),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .style(
            6,
            NodeInput {
                size: Size::new(PreferredSize::px(40.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(2, Size::new(0.0, 40.0))
        .measure(3, Size::new(20.0, 40.0));
    let sizes = fri08_c02_track_sizes(
        &tree,
        crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(scalar(80.0), scalar(40.0)),
        Fri08C02TrackAxis::Columns,
        2,
    );
    assert_eq!(sizes, [60.0, 20.0]);
}

fn assert_fri08_c06_collapsed_gutter_carrier<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for (collapsed, expected) in [
        (vec![false, true, false], vec![scalar(10.0), S::ZERO]),
        (
            vec![false, true, true, false, true, false],
            vec![scalar(10.0), S::ZERO, S::ZERO, scalar(10.0), S::ZERO],
        ),
        (
            vec![true, true, false, false],
            vec![S::ZERO, S::ZERO, scalar(10.0)],
        ),
        (
            vec![false, false, true, true],
            vec![scalar(10.0), S::ZERO, S::ZERO],
        ),
        (vec![true, true, true], vec![S::ZERO, S::ZERO]),
    ] {
        let gutters = OrdinaryGridAxisGuttersOf::new(collapsed.len(), &collapsed, scalar(10.0));
        assert_eq!(gutters.gutter_after(), expected);
        assert_eq!(
            gutters.active_gap_total(),
            expected
                .iter()
                .copied()
                .fold(S::ZERO, |sum, gutter| sum + gutter)
        );
    }
}

#[test]
fn fri08_c06_collapsed_gutter_carrier_retains_one_gap_per_interior_run_for_both_scalars() {
    assert_fri08_c06_collapsed_gutter_carrier::<f32>();
    assert_fri08_c06_collapsed_gutter_carrier::<f64>();

    let reversed = OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
        4,
        &[false, true, false, false],
        &[true, false, true],
        &[10.0_f64, 0.0, 30.0],
    )
    .reversed();
    assert_eq!(reversed.collapsed(), &[false, false, true, false]);
    assert_eq!(reversed.gutter_after(), &[30.0, 10.0, 0.0]);
}

fn assert_fri08_c06_collapsed_gutter_public_axis<S: LayoutScalar>(axis: Fri08C02TrackAxis) {
    let scalar = S::from_f64;
    let (size, columns, rows, first_column, first_row, third_column, third_row) = match axis {
        Fri08C02TrackAxis::Columns => (
            Size::new(
                PreferredSizeOf::px(scalar(140.0)),
                PreferredSizeOf::px(scalar(20.0)),
            ),
            vec![fri08_c02_auto_fit_repeat()],
            vec![TrackComponentOf::px(scalar(20.0))],
            GridPlacement::try_line(1).expect("first repeated column"),
            GridPlacement::try_line(1).expect("single row"),
            GridPlacement::try_line(3).expect("third repeated column"),
            GridPlacement::try_line(1).expect("single row"),
        ),
        Fri08C02TrackAxis::Rows => (
            Size::new(
                PreferredSizeOf::px(scalar(20.0)),
                PreferredSizeOf::px(scalar(140.0)),
            ),
            vec![TrackComponentOf::px(scalar(20.0))],
            vec![fri08_c02_auto_fit_repeat()],
            GridPlacement::try_line(1).expect("single column"),
            GridPlacement::try_line(1).expect("first repeated row"),
            GridPlacement::try_line(1).expect("single column"),
            GridPlacement::try_line(3).expect("third repeated row"),
        ),
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size,
                grid_template_columns: columns,
                grid_template_rows: rows,
                gap: Size::new(LengthOf::px(scalar(10.0)), LengthOf::px(scalar(10.0))),
                justify_content: Some(AlignContent::Center),
                align_content: Some(AlignContent::Center),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                grid_column: first_column,
                grid_row: first_row,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: third_column,
                grid_row: third_row,
                ..NodeInputOf::default()
            },
        );
    let viewport = match axis {
        Fri08C02TrackAxis::Columns => Size::new(scalar(140.0), scalar(20.0)),
        Fri08C02TrackAxis::Rows => Size::new(scalar(20.0), scalar(140.0)),
    };
    let first = fri08_c02_auto_fit_output(&tree, viewport, 2);
    let third = fri08_c02_auto_fit_output(&tree, viewport, 3);
    let actual = match axis {
        Fri08C02TrackAxis::Columns => (
            first.location.x,
            first.size.width,
            third.location.x,
            third.size.width,
        ),
        Fri08C02TrackAxis::Rows => (
            first.location.y,
            first.size.height,
            third.location.y,
            third.size.height,
        ),
    };
    assert_eq!(
        actual,
        (scalar(25.0), scalar(40.0), scalar(75.0), scalar(40.0))
    );
}

#[test]
fn fri08_c06_collapsed_gutter_public_layout_matches_on_both_axes_and_scalars() {
    for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
        assert_fri08_c06_collapsed_gutter_public_axis::<f32>(axis);
        assert_fri08_c06_collapsed_gutter_public_axis::<f64>(axis);
    }
}

#[test]
fn fri08_c02_auto_fit_all_empty_repetitions_and_adjacent_gutters_collapse() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_lines(1, 4).expect("all retained lines"),
                grid_row: GridPlacement::try_lines(1, 2).expect("single row"),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );

    let output = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 2);
    assert_eq!(output.location.x, 70.0);
    assert_eq!(output.size.width, 0.0);
}

#[test]
fn fri08_c02_auto_fit_interior_collapse_retains_one_coincident_gutter() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("first repetition"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(3).expect("third repetition"),
                ..NodeInput::DEFAULT
            },
        );

    let first = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 2);
    let third = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 3);
    assert_eq!((first.location.x, first.size.width), (25.0, 40.0));
    assert_eq!((third.location.x, third.size.width), (75.0, 40.0));
}

#[test]
fn fri08_c02_auto_fit_public_intrinsic_span_counts_only_active_boundary_gutters() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                size: Size::new(PreferredSize::AUTO, PreferredSize::px(20.0)),
                max_size: Size::new(crate::MaxSize::px(140.0), crate::MaxSize::NONE),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Start),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_lines(1, 3)
                    .expect("span across the two active repetitions"),
                grid_row: GridPlacement::try_line(1).expect("single row"),
                ..NodeInput::DEFAULT
            },
        );

    let batch = compute_layout(&tree, 1, {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(140.0).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(20.0).unwrap()),
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
    .expect("public intrinsic auto-fit layout");
    let grid = fri08_c01_placement_output(&batch, 1);
    let spanning = fri08_c01_placement_output(&batch, 2);

    assert_eq!(grid.size.width, 90.0);
    assert_eq!(spanning.size.width, 90.0);
}

#[test]
fn fri08_c02_auto_fit_public_flex_uses_active_gutter_total_for_free_space() {
    let flex_repeat = TrackComponent::Repeat(
        TrackRepetition::auto_fit_components(vec![TrackComponent::minmax(
            MinTrackSizing::px(40.0),
            MaxTrackSizing::flex(
                TrackFlexFactor::try_new(1.0).expect("finite auto-fit flex factor"),
            ),
        )])
        .expect("valid flexible auto-fit repetition"),
    );
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![flex_repeat],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Start),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("first repeated track"),
                grid_row: GridPlacement::try_line(1).expect("single row"),
                ..NodeInput::DEFAULT
            },
        );

    let output = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 2);
    assert_eq!((output.location.x, output.size.width), (0.0, 140.0));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fri06C07HeightFamily {
    GridLanes { overflow_hidden: bool },
    SubgridMinContent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fri06C07HeightRow {
    source: &'static str,
    variant: &'static str,
    family: Fri06C07HeightFamily,
    box_sizing: BoxSizing,
    direction: Direction,
}

const FRI06_C07_HEIGHT_ROWS: [Fri06C07HeightRow; 12] = [
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_normal_packing",
        variant: "border_box_ltr",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: false,
        },
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
    },
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_normal_packing",
        variant: "content_box_ltr",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: false,
        },
        box_sizing: BoxSizing::ContentBox,
        direction: Direction::Ltr,
    },
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_normal_packing",
        variant: "border_box_rtl",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: false,
        },
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Rtl,
    },
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_normal_packing",
        variant: "content_box_rtl",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: false,
        },
        box_sizing: BoxSizing::ContentBox,
        direction: Direction::Rtl,
    },
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_overflow_hidden_packing",
        variant: "border_box_ltr",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: true,
        },
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
    },
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_overflow_hidden_packing",
        variant: "content_box_ltr",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: true,
        },
        box_sizing: BoxSizing::ContentBox,
        direction: Direction::Ltr,
    },
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_overflow_hidden_packing",
        variant: "border_box_rtl",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: true,
        },
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Rtl,
    },
    Fri06C07HeightRow {
        source: "grid_lanes_not_inhibited_overflow_hidden_packing",
        variant: "content_box_rtl",
        family: Fri06C07HeightFamily::GridLanes {
            overflow_hidden: true,
        },
        box_sizing: BoxSizing::ContentBox,
        direction: Direction::Rtl,
    },
    Fri06C07HeightRow {
        source: "subgrid_auto_track_sizing_min_content_text_runs",
        variant: "border_box_ltr",
        family: Fri06C07HeightFamily::SubgridMinContent,
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
    },
    Fri06C07HeightRow {
        source: "subgrid_auto_track_sizing_min_content_text_runs",
        variant: "content_box_ltr",
        family: Fri06C07HeightFamily::SubgridMinContent,
        box_sizing: BoxSizing::ContentBox,
        direction: Direction::Ltr,
    },
    Fri06C07HeightRow {
        source: "subgrid_auto_track_sizing_min_content_text_runs",
        variant: "border_box_rtl",
        family: Fri06C07HeightFamily::SubgridMinContent,
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Rtl,
    },
    Fri06C07HeightRow {
        source: "subgrid_auto_track_sizing_min_content_text_runs",
        variant: "content_box_rtl",
        family: Fri06C07HeightFamily::SubgridMinContent,
        box_sizing: BoxSizing::ContentBox,
        direction: Direction::Rtl,
    },
];

fn fri06_c07_height_grid_lanes_tree<S: LayoutScalar>(
    row: Fri06C07HeightRow,
    overflow_hidden: bool,
) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let overflow = if overflow_hidden {
        ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden)
            .expect("hidden overflow is canonical")
    } else {
        ComputedOverflow::VISIBLE
    };
    let root = NodeInputOf {
        display: Display::GridLanes,
        box_sizing: row.box_sizing,
        direction: row.direction,
        overflow,
        size: Size::new(PreferredSizeOf::px(scalar(120.0)), PreferredSizeOf::AUTO),
        grid_template_columns: vec![
            TrackComponentOf::px(scalar(60.0)),
            TrackComponentOf::px(scalar(60.0)),
        ],
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        justify_items: Some(AlignItems::Start),
        align_items: Some(AlignItems::Start),
        ..NodeInputOf::default()
    };
    let child = |height| NodeInputOf {
        display: Display::Block,
        box_sizing: row.box_sizing,
        direction: row.direction,
        size: Size::new(
            PreferredSizeOf::px(scalar(60.0)),
            PreferredSizeOf::px(height),
        ),
        ..NodeInputOf::default()
    };
    PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(0, root)
        .style(1, child(scalar(60.0)))
        .style(2, child(scalar(30.0)))
        .style(3, child(scalar(30.0)))
}

fn fri06_c07_height_subgrid_tree<S: LayoutScalar>(row: Fri06C07HeightRow) -> PublicLayoutTreeOf<S> {
    let fixture_root = NodeInputOf {
        display: Display::Block,
        box_sizing: row.box_sizing,
        direction: row.direction,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(100.0)),
            PreferredSizeOf::px(S::from_f64(100.0)),
        ),
        ..NodeInputOf::default()
    };
    let outer = NodeInputOf {
        // The source min-content text-runs fixture uses ordinary Grid.
        display: Display::Grid,
        box_sizing: row.box_sizing,
        direction: row.direction,
        size: Size::new(PreferredSizeOf::MIN_CONTENT, PreferredSizeOf::AUTO),
        ..NodeInputOf::default()
    };
    let subgrid = NodeInputOf {
        display: Display::Grid,
        box_sizing: row.box_sizing,
        direction: row.direction,
        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
        grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
        grid_column: GridPlacement::try_lines(1, -1).expect("full column subgrid span is valid"),
        grid_row: GridPlacement::try_lines(1, -1).expect("full row subgrid span is valid"),
        ..NodeInputOf::default()
    };
    let text_container = NodeInputOf {
        display: Display::Block,
        box_sizing: row.box_sizing,
        direction: row.direction,
        ..NodeInputOf::default()
    };
    // This height-dependent Grid experiment consumes actual numeric inline
    // boxes: four 25px line allocations with declared border widths and breaks.
    // Font selection/shaping is independently covered by prepared Text tests.
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5, 6, 7])
        .style(0, fixture_root)
        .style(1, outer)
        .style(2, subgrid)
        .style(3, text_container);
    for (index, width) in [25.0, 100.0, 50.0, 75.0].into_iter().enumerate() {
        let node = 4 + index as u32;
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::px(S::from_f64(25.0)),
                ),
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(u8::from(row.direction == Direction::Rtl)).unwrap(),
                    if index == 3 {
                        InlineBreakOpportunity::Prohibited
                    } else {
                        InlineBreakOpportunity::Allowed
                    },
                )),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn assert_fri06_c07_height_grid_lanes<S: LayoutScalar>(
    row: Fri06C07HeightRow,
    overflow_hidden: bool,
) {
    let scalar = S::from_f64;
    let tree = fri06_c07_height_grid_lanes_tree(row, overflow_hidden);
    let batch = compute_layout(&tree, 0, {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(scalar(120.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        );
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
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("grid-lanes packing computes through the public front door");
    let root = readback::single_box_output(&batch, 0, Phase::Unrounded);

    assert_eq!(root.size, Size::new(scalar(120.0), scalar(60.0)), "{row:?}");
    assert_eq!(
        root.content_size,
        Size::new(scalar(120.0), scalar(60.0)),
        "{row:?}"
    );
    let expected_x = if row.direction == Direction::Ltr {
        [0.0, 60.0, 60.0]
    } else {
        [60.0, 0.0, 0.0]
    };
    for ((node, x), y) in [1, 2, 3].into_iter().zip(expected_x).zip([0.0, 0.0, 30.0]) {
        assert_eq!(
            readback::single_box_output(&batch, node, Phase::Unrounded).location,
            Point::new(scalar(x), scalar(y)),
            "{row:?} child {node}"
        );
    }
}

fn assert_fri06_c07_height_subgrid<S: LayoutScalar>(row: Fri06C07HeightRow) {
    let scalar = S::from_f64;
    let tree = fri06_c07_height_subgrid_tree(row);
    let batch = compute_layout(&tree, 0, {
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
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("subgrid min-content layout computes through the public front door");
    let grid = readback::single_box_output(&batch, 1, Phase::Unrounded);

    assert_eq!(
        grid.size,
        Size::new(scalar(100.0), scalar(100.0)),
        "{row:?}"
    );
    assert_eq!(
        grid.content_size,
        Size::new(scalar(100.0), scalar(100.0)),
        "{row:?}"
    );
    let expected_x = if row.direction == Direction::Ltr {
        [0.0, 0.0, 0.0, 0.0]
    } else {
        [75.0, 0.0, 50.0, 25.0]
    };
    for (((node, x), y), width) in [4, 5, 6, 7]
        .into_iter()
        .zip(expected_x)
        .zip([0.0, 25.0, 50.0, 75.0])
        .zip([25.0, 100.0, 50.0, 75.0])
    {
        let output = readback::single_box_output(&batch, node, Phase::Unrounded);
        assert_eq!(
            output.location,
            Point::new(scalar(x), scalar(y)),
            "{row:?} atomic {node}"
        );
        assert_eq!(
            output.size,
            Size::new(scalar(width), scalar(25.0)),
            "{row:?} atomic {node}"
        );
    }
}

fn assert_fri06_c07_height_rows<S: LayoutScalar>() {
    assert_eq!(FRI06_C07_HEIGHT_ROWS.len(), 12);
    let unique_rows = FRI06_C07_HEIGHT_ROWS
        .iter()
        .map(|row| (row.source, row.variant))
        .collect::<HashSet<_>>();
    assert_eq!(unique_rows.len(), 12);

    for row in FRI06_C07_HEIGHT_ROWS {
        match row.family {
            Fri06C07HeightFamily::GridLanes { overflow_hidden } => {
                assert_fri06_c07_height_grid_lanes::<S>(row, overflow_hidden);
            }
            Fri06C07HeightFamily::SubgridMinContent => {
                assert_fri06_c07_height_subgrid::<S>(row);
            }
        }
    }
}

#[test]
fn fri06_c07_height_exact_twelve_rows_preserve_packing_and_intrinsic_block_size() {
    assert_fri06_c07_height_rows::<f32>();
    assert_fri06_c07_height_rows::<f64>();
}

fn assert_grid_track_sum_overflow_owner<S: LayoutScalar>(display: Display) {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(7, [])
        .style(
            7,
            NodeInputOf {
                display,
                size: size.map(PreferredSizeOf::px),
                grid_template_columns: vec![
                    TrackComponentOf::px(largest),
                    TrackComponentOf::px(largest),
                ],
                grid_template_rows: if display == Display::Grid {
                    vec![TrackComponentOf::AUTO]
                } else {
                    Vec::new()
                },
                ..NodeInputOf::default()
            },
        );
    let error = compute_grid(
        &mut tree.borrow_compute(),
        7,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            size.map(Some),
            size.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                size.map(|value| {
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(value).unwrap()),
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
    )
    .expect_err("overflowing track subject geometry must fail");

    // Both authored tracks are finite; their sum overflows in the numeric
    // owner before any child/scroll geometry exists. Preserve that owning
    // identity and the specific arithmetic failure rather than a later site.
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::InvalidNumeric { value })
            if !value.is_finite() && *value > S::ZERO
    ));
}

#[test]
fn grid_track_sum_overflow_preserves_owner_identity_both_scalars() {
    for display in [Display::Grid, Display::GridLanes] {
        assert_grid_track_sum_overflow_owner::<f32>(display);
        assert_grid_track_sum_overflow_owner::<f64>(display);
    }
}

fn fri05_c05_grid_contribution_nested(
    display: Display,
    overflow: ComputedOverflow,
    child_size: Size<f32>,
) -> ComputeOutput {
    let mut tree = OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display,
                size: Size::ZERO.map(PreferredSize::px),
                grid_template_columns: vec![TrackComponent::px(0.0)],
                grid_template_rows: vec![TrackComponent::px(0.0)],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow,
                size: child_size.map(PreferredSize::px),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(30.0)),
                ..NodeInput::default()
            },
        );

    compute_grid(
        &mut tree.borrow_compute(),
        0,
        fri05_c05_grid_sizing_input(Size::splat(Some(0.0))),
    )
    .expect("nested grid contribution computes")
}

#[test]
fn fri05_c05_grid_contribution_zero_axis_visible_descendants_and_traps_are_independent() {
    for display in [Display::Grid, Display::GridLanes] {
        for (overflow, child_size, expected) in [
            (
                computed_overflow(Overflow::Visible, Overflow::Clip),
                Size::new(0.0, 5.0),
                Size::new(20.0, 0.0),
            ),
            (
                computed_overflow(Overflow::Clip, Overflow::Visible),
                Size::new(5.0, 0.0),
                Size::new(0.0, 30.0),
            ),
            (
                computed_overflow(Overflow::Hidden, Overflow::Scroll),
                Size::new(0.0, 5.0),
                Size::ZERO,
            ),
            (
                computed_overflow(Overflow::Auto, Overflow::Auto),
                Size::new(5.0, 0.0),
                Size::ZERO,
            ),
        ] {
            let output = fri05_c05_grid_contribution_nested(display, overflow, child_size);
            let expected = if display == Display::GridLanes && child_size.width == 0.0 {
                expected.max(Size::new(0.0, child_size.height))
            } else {
                expected
            };
            assert_eq!(output.content_size, expected, "{display:?} {overflow:?}");
        }
    }
}

fn track_flex<S: LayoutScalar>(value: S) -> TrackSizingOf<S> {
    TrackSizingOf::flex(TrackFlexFactorOf::try_new(value).expect("valid test track flex factor"))
}

#[test]
fn fri04_c04_grid_dispatch_supported_calc_size_and_intrinsic_geometry_in_both_axes() {
    for display in [
        Display::Grid,
        Display::InlineGrid,
        Display::GridLanes,
        Display::InlineGridLanes,
    ] {
        let empty_height = if display.establishes_grid_lanes_formatting_context() {
            0.0
        } else {
            20.0
        };
        let intrinsic_size = Size::new(30.0, empty_height);
        let mut auto_tree = OracleTree::new().children(0, []).style(
            0,
            NodeInput {
                display,
                size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
                grid_template_columns: vec![TrackComponent::px(30.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::default()
            },
        );
        let output = compute_grid(
            &mut auto_tree.borrow_compute(),
            0,
            fri04_c04_grid_dispatch_input(Size::NONE),
        )
        .expect("supported preferred auto grid values resolve");
        assert_eq!(output.size, intrinsic_size, "{display:?} auto");

        let mut numeric_tree = OracleTree::new().children(0, []).style(
            0,
            NodeInput {
                display,
                size: Size::new(PreferredSize::px(70.0), PreferredSize::px(50.0)),
                ..NodeInput::default()
            },
        );
        let output = compute_grid(
            &mut numeric_tree.borrow_compute(),
            0,
            fri04_c04_grid_dispatch_input(Size::new(Some(200.0), Some(160.0))),
        )
        .expect("supported preferred numeric grid values resolve");
        assert_eq!(output.size, Size::new(70.0, 50.0), "{display:?} numeric");

        for (basis, calculation, expected) in [
            (
                PreferredSizeCalcBasis::Any,
                CalcSizeCalculation::from_coefficients(20.0, 0.5, 0.0)
                    .expect("finite Any calculation"),
                Size::new(120.0, 100.0),
            ),
            (
                PreferredSizeCalcBasis::FullPercentage,
                CalcSizeCalculation::from_coefficients(10.0, 0.0, 0.5)
                    .expect("finite FullPercentage calculation"),
                Size::new(110.0, 90.0),
            ),
        ] {
            let mut calc_tree = OracleTree::new().children(0, []).style(
                0,
                NodeInput {
                    display,
                    size: Size::new(
                        PreferredSize::calc_size(basis, calculation.clone())
                            .expect("valid calc-size width"),
                        PreferredSize::calc_size(basis, calculation)
                            .expect("valid calc-size height"),
                    ),
                    ..NodeInput::default()
                },
            );
            let output = compute_grid(
                &mut calc_tree.borrow_compute(),
                0,
                fri04_c04_grid_dispatch_input(Size::new(Some(200.0), Some(160.0))),
            )
            .expect("supported grid calc-size values resolve");
            assert_eq!(output.size, expected, "{display:?} {basis:?}");
        }

        for intrinsic in [PreferredSize::MIN_CONTENT, PreferredSize::MAX_CONTENT] {
            let mut intrinsic_tree = OracleTree::new().children(0, []).style(
                0,
                NodeInput {
                    display,
                    size: Size::new(intrinsic.clone(), intrinsic),
                    grid_template_columns: vec![TrackComponent::px(30.0)],
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    ..NodeInput::default()
                },
            );
            let output = compute_grid(
                &mut intrinsic_tree.borrow_compute(),
                0,
                fri04_c04_grid_dispatch_input(Size::NONE),
            )
            .expect("supported preferred intrinsic grid values resolve");
            assert_eq!(output.size, intrinsic_size, "{display:?} intrinsic");
        }
    }
}

#[test]
fn fri04_c03_grid_track_nested_track_breadths_and_fit_content_use_complete_programs() {
    let fixed = TrackSizing::calculation(fri04_c03_grid_track_nested(20.0, 60.0, 80.0));
    assert_eq!(track_base_size(&fixed, None, 11.0), 60.0);
    assert_eq!(
        track_growth_limit(
            &fixed,
            None,
            GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None
            },
            GridTrackContributions {
                minimum: 11.0,
                min_content: 11.0,
                max_content: 11.0
            }
        ),
        Some(60.0)
    );

    let negative = TrackSizing::calculation(fri04_c03_grid_track_nested(-30.0, -10.0, -5.0));
    assert_eq!(track_base_size(&negative, None, 11.0), 0.0);

    let fit = TrackSizing::fit_content(fri04_c03_grid_track_nested(20.0, 60.0, 80.0));
    assert_eq!(
        {
            // Each listed row is one authored single-track item, including zero-sized items.
            let tracks = &[fit];
            let items = [single_track_item(0, 20.0, 20.0, 100.0)];
            let phases = resolve_grid_contribution_rows(
                SpanningTrackContext {
                    tracks,
                    basis: None,
                    available: GridTrackSizingConstraint {
                        operation: SizingOperation::MaxContent,
                        available_extent: None,
                    },
                    gap: 0.0,
                    gutters: None,
                    base_floors: &[],
                },
                &items,
            )
            .expect("authored item incidence resolves checked phases");
            resolve_tracks_from_phases(ResolvedAxisTrackInput {
                tracks,
                phases: &phases,
                items: &items,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                flex_bounds: GridTrackFinishingBounds::NONE,
                stretch_size: None,
                gap: 0.0,
                alignment: AlignContent::Start,
                gutters: None,
            })
            .expect("authored track phases finish without losing item spans")
        },
        [60.0]
    );

    let dependent =
        TrackSizing::calculation(fri04_c03_grid_track_percentage_nested(20.0, 0.5, 80.0));
    assert_eq!(track_base_size(&dependent, None, 11.0), 0.0);
    assert_eq!(track_base_size(&dependent, Some(100.0), 11.0), 50.0);

    let dependent_fit =
        TrackSizing::fit_content(fri04_c03_grid_track_percentage_nested(20.0, 0.6, 80.0));
    assert_eq!(
        {
            // Each listed row is one authored single-track item, including zero-sized items.
            let tracks = core::slice::from_ref(&dependent_fit);
            let items = [single_track_item(0, 20.0, 20.0, 100.0)];
            let phases = resolve_grid_contribution_rows(
                SpanningTrackContext {
                    tracks,
                    basis: None,
                    available: GridTrackSizingConstraint {
                        operation: SizingOperation::MaxContent,
                        available_extent: None,
                    },
                    gap: 0.0,
                    gutters: None,
                    base_floors: &[],
                },
                &items,
            )
            .expect("authored item incidence resolves checked phases");
            resolve_tracks_from_phases(ResolvedAxisTrackInput {
                tracks,
                phases: &phases,
                items: &items,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                flex_bounds: GridTrackFinishingBounds::NONE,
                stretch_size: None,
                gap: 0.0,
                alignment: AlignContent::Start,
                gutters: None,
            })
            .expect("authored track phases finish without losing item spans")
        },
        [100.0]
    );
    let definite_fit = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &[dependent_fit];
        let items = [single_track_item(0, 20.0, 20.0, 100.0)];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: Some(100.0),
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(100.0),
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(100.0),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(100.0),
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };
    assert!(
        (definite_fit[0] - 60.0).abs() <= 0.000_01,
        "definite nested fit-content limit: {}",
        definite_fit[0]
    );
}

#[test]
fn fri04_c03_grid_track_nested_classification_controls_intrinsic_definite_space_and_floor() {
    let independent = TrackSizing::calculation(fri04_c03_grid_track_nested(20.0, 40.0, 60.0));
    assert!(!track_has_percent_sizing(&independent));
    assert_eq!(independent.min.definite(None), Some(40.0));
    assert_eq!(track_min_floor_space(&independent), 40.0);
    assert_eq!(
        intrinsic_span_definite_track_space(core::slice::from_ref(&independent)),
        40.0
    );
    assert_eq!(
        intrinsic_span_minimum_floor_space(core::slice::from_ref(&independent)),
        40.0
    );

    let dependent = TrackSizing::calculation(
        SizingCalculation::max(vec![
            fri04_c03_grid_track_value(40.0),
            SizingCalculation::value(
                LengthPercentageOf::from_percent_fraction(0.1).expect("finite percentage"),
            ),
        ])
        .expect("nested maximum is nonempty"),
    );
    assert!(track_has_percent_sizing(&dependent));
    assert_eq!(dependent.min.definite(None), None);
    assert_eq!(track_min_floor_space(&dependent), 0.0);
}

fn fake_leaf_error(
    node: u32,
    error: LayoutError<(), core::convert::Infallible>,
) -> LayoutError<u32> {
    LayoutError::new(
        LayoutErrorSite::Node(node),
        error.operation(),
        match error.kind() {
            LayoutErrorKind::InvalidInput(value) => LayoutErrorKind::InvalidInput(value.clone()),
            LayoutErrorKind::MissingContext(value) => LayoutErrorKind::MissingContext(*value),
            LayoutErrorKind::UnsupportedCapability(value) => {
                LayoutErrorKind::UnsupportedCapability(*value)
            }
            LayoutErrorKind::InternalInvariant(value) => LayoutErrorKind::InternalInvariant(*value),
            other => panic!("infallible scalar leaf returned an unexpected cause: {other:?}"),
        },
    )
}

#[test]
fn owner_to_current_placement_map_composes_two_boundaries_by_track_and_role() {
    let inherited = inherit_subgrid_tracks::<f32>(SubgridTrackInheritanceInput {
        parent_tracks: &[40.0, 50.0, 60.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(20.0),
    })
    .unwrap();
    assert_eq!(inherited.final_tracks, vec![35.0, 40.0, 55.0]);
    assert_eq!(inherited.final_boundary_gutters, vec![20.0, 20.0]);
    let child_geometry = UsedGridAxisGeometryOf::new(
        inherited.final_tracks.clone(),
        inherited.collapsed.clone(),
        inherited.resolved_subgrid_gap,
    );
    assert_eq!(child_geometry.line_offsets(), &[0.0, 55.0, 115.0, 170.0]);
    let nested = inherit_subgrid_tracks::<f32>(SubgridTrackInheritanceInput {
        parent_tracks: &inherited.final_tracks,
        parent_span: GridTrackSpan::new(2, 4),
        reversed: false,
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_gap: 20.0,
        subgrid_gap: ResolvedSubgridGap::Length(30.0),
    })
    .unwrap();
    assert_eq!(nested.final_tracks, vec![35.0, 50.0]);
    assert_eq!(nested.final_boundary_gutters, vec![30.0]);
    let nested_geometry = UsedGridAxisGeometryOf::new(
        nested.final_tracks.clone(),
        nested.collapsed.clone(),
        nested.resolved_subgrid_gap,
    );
    assert_eq!(nested_geometry.line_offsets(), &[0.0, 65.0, 115.0]);
    // Owner starts0/50/110 and ends40/100/170. Child starts0/55/115,
    // ends35/95/170. The nested area starts55 in that child, giving actual
    // nested starts55/120 and ends90/170 in the same physical coordinates.
    // First frames are negative starts; Last frames are positive ends.
    let identity = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        3,
    );
    let first = identity
        .compose(OwnerToCurrentPlacementBoundaryInput {
            parent_grid: 1,
            current_grid: 2,
            parent_axis: GridAxisKind::Row,
            current_axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            parent_progression: PhysicalProgression::Increasing,
            current_progression: PhysicalProgression::Increasing,
            parent_span: GridTrackSpan::new(0, 3),
            reversed: false,
            parent_first_frame_origins: &[0.0, -50.0, -110.0],
            parent_last_frame_origins: &[40.0, 100.0, 170.0],
            current_first_frame_origins: &[0.0, -55.0, -115.0],
            current_last_frame_origins: &[35.0, 95.0, 170.0],
            inherited: true,
        })
        .unwrap();
    let second = first
        .compose(OwnerToCurrentPlacementBoundaryInput {
            parent_grid: 2,
            current_grid: 3,
            parent_axis: GridAxisKind::Row,
            current_axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            parent_progression: PhysicalProgression::Increasing,
            current_progression: PhysicalProgression::Increasing,
            parent_span: GridTrackSpan::new(1, 3),
            reversed: false,
            parent_first_frame_origins: &[0.0, -55.0, -115.0],
            parent_last_frame_origins: &[35.0, 95.0, 170.0],
            current_first_frame_origins: &[-55.0, -120.0],
            current_last_frame_origins: &[90.0, 170.0],
            inherited: true,
        })
        .unwrap();
    assert_eq!(second.owner(), 1);
    assert_eq!(second.current_grid(), 3);
    assert_eq!(second.boundary_count(), 2);
    assert_eq!(second.owner_track_for_local(0), Some(1));
    assert_eq!(second.owner_track_for_local(1), Some(2));
    for (
        role,
        local,
        owner_track,
        owner_start,
        owner_end,
        current_start,
        current_end,
        shift,
        point,
    ) in [
        (
            AncestorBaselineRole::First,
            0,
            1,
            50.0,
            100.0,
            55.0,
            90.0,
            -5.0,
            67.0,
        ),
        (
            AncestorBaselineRole::First,
            1,
            2,
            110.0,
            170.0,
            120.0,
            170.0,
            -10.0,
            127.0,
        ),
        (
            AncestorBaselineRole::Last,
            0,
            1,
            50.0,
            100.0,
            55.0,
            90.0,
            -10.0,
            83.0,
        ),
        (
            AncestorBaselineRole::Last,
            1,
            2,
            110.0,
            170.0,
            120.0,
            170.0,
            0.0,
            153.0,
        ),
    ] {
        let group = AncestorBaselineGroup::reduce(
            1_u32,
            GridAxisKind::Row,
            PhysicalAxis::Vertical,
            crate::GridBaselineReference::Physical,
            3,
            [inherited_placement_member(
                91,
                GridAxisKind::Row,
                role,
                owner_track,
                17.0,
            )],
        );
        let placement = InheritedCurrentGridBaselinePlacement::try_derive(
            &group,
            InheritedCurrentGridBaselinePlacementInput {
                reference: crate::GridBaselineReference::Physical,
                axis: GridAxisKind::Row,
                physical_axis: PhysicalAxis::Vertical,
                mapping: second.clone(),
                direct_witness: CurrentGridDirectWitness::new(
                    3,
                    4,
                    GridAxisKind::Row,
                    GridTrackSpan::new(local, local + 1),
                    role,
                    PhysicalProgression::Increasing,
                    PhysicalProgression::Increasing,
                ),
                current_grid: 3,
                item: 4,
            },
        )
        .unwrap();
        assert_eq!(placement.selected_local_track(), local);
        assert_eq!(placement.immutable_owner_target(), 17.0);
        assert_eq!(placement.frame_translation(), shift);
        let (owner_point, current_point) = match role {
            AncestorBaselineRole::First => (
                owner_start + 17.0,
                current_start + placement.translated_target(),
            ),
            AncestorBaselineRole::Last => (
                owner_end - 17.0,
                current_end - placement.translated_target(),
            ),
        };
        assert_eq!(owner_point, point);
        assert_eq!(current_point, point);
    }
}

#[test]
fn owner_to_current_placement_map_preserves_points_through_mbp_and_gutters() {
    let inherited = inherit_subgrid_tracks::<f32>(SubgridTrackInheritanceInput {
        parent_tracks: &[30.0, 40.0, 60.0],
        parent_span: GridTrackSpan::new(1, 4),
        reversed: false,
        start_mbp: 3.0,
        end_mbp: 4.0,
        parent_gap: 10.0,
        subgrid_gap: ResolvedSubgridGap::Length(20.0),
    })
    .unwrap();
    assert_eq!(inherited.final_tracks, vec![22.0, 30.0, 51.0]);
    assert_eq!(inherited.final_boundary_gutters, vec![20.0, 20.0]);
    let child_geometry = UsedGridAxisGeometryOf::new(
        inherited.final_tracks.clone(),
        inherited.collapsed.clone(),
        inherited.resolved_subgrid_gap,
    );
    assert_eq!(child_geometry.line_offsets(), &[0.0, 42.0, 92.0, 143.0]);
    // The child content starts3 in the parent. Its actual track starts are
    //3/45/95 and ends25/75/146; the parent's are0/40/90 and30/80/150.
    let map = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        3,
    )
    .compose(OwnerToCurrentPlacementBoundaryInput {
        parent_grid: 1,
        current_grid: 2,
        parent_axis: GridAxisKind::Row,
        current_axis: GridAxisKind::Row,
        physical_axis: PhysicalAxis::Vertical,
        parent_progression: PhysicalProgression::Increasing,
        current_progression: PhysicalProgression::Increasing,
        parent_span: GridTrackSpan::new(0, 3),
        reversed: false,
        parent_first_frame_origins: &[0.0, -40.0, -90.0],
        parent_last_frame_origins: &[30.0, 80.0, 150.0],
        current_first_frame_origins: &[-3.0, -45.0, -95.0],
        current_last_frame_origins: &[25.0, 75.0, 146.0],
        inherited: true,
    })
    .unwrap();
    assert_eq!(map.owner(), 1);
    assert_eq!(map.current_grid(), 2);
    assert_eq!(map.boundary_count(), 1);
    for (role, track, owner_start, owner_end, current_start, current_end, shift) in [
        (AncestorBaselineRole::First, 0, 0.0, 30.0, 3.0, 25.0, -3.0),
        (AncestorBaselineRole::First, 1, 40.0, 80.0, 45.0, 75.0, -5.0),
        (
            AncestorBaselineRole::First,
            2,
            90.0,
            150.0,
            95.0,
            146.0,
            -5.0,
        ),
        (AncestorBaselineRole::Last, 0, 0.0, 30.0, 3.0, 25.0, -5.0),
        (AncestorBaselineRole::Last, 1, 40.0, 80.0, 45.0, 75.0, -5.0),
        (
            AncestorBaselineRole::Last,
            2,
            90.0,
            150.0,
            95.0,
            146.0,
            -4.0,
        ),
    ] {
        let group = AncestorBaselineGroup::reduce(
            1_u32,
            GridAxisKind::Row,
            PhysicalAxis::Vertical,
            crate::GridBaselineReference::Physical,
            3,
            [inherited_placement_member(
                91,
                GridAxisKind::Row,
                role,
                track,
                17.0,
            )],
        );
        let placement = InheritedCurrentGridBaselinePlacement::try_derive(
            &group,
            InheritedCurrentGridBaselinePlacementInput {
                reference: crate::GridBaselineReference::Physical,
                axis: GridAxisKind::Row,
                physical_axis: PhysicalAxis::Vertical,
                mapping: map.clone(),
                direct_witness: CurrentGridDirectWitness::new(
                    2,
                    4,
                    GridAxisKind::Row,
                    GridTrackSpan::new(track, track + 1),
                    role,
                    PhysicalProgression::Increasing,
                    PhysicalProgression::Increasing,
                ),
                current_grid: 2,
                item: 4,
            },
        )
        .unwrap();
        assert_eq!(map.owner_track_for_local(track), Some(track));
        assert_eq!(placement.immutable_owner_target(), 17.0);
        assert_eq!(placement.frame_translation(), shift);
        let (owner_point, current_point) = match role {
            AncestorBaselineRole::First => (
                owner_start + 17.0,
                current_start + placement.translated_target(),
            ),
            AncestorBaselineRole::Last => (
                owner_end - 17.0,
                current_end - placement.translated_target(),
            ),
        };
        assert_eq!(
            current_point, owner_point,
            "track{track}/{role:?} preserves the actual physical point"
        );
    }
}

#[test]
fn owner_to_current_placement_map_rejects_track_cardinality_mismatch() {
    let identity = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        PhysicalProgression::Increasing,
        2,
    );
    let result = identity.compose(OwnerToCurrentPlacementBoundaryInput {
        parent_grid: 1,
        current_grid: 2,
        parent_axis: GridAxisKind::Row,
        current_axis: GridAxisKind::Row,
        physical_axis: PhysicalAxis::Vertical,
        parent_progression: PhysicalProgression::Increasing,
        current_progression: PhysicalProgression::Increasing,
        parent_span: GridTrackSpan::new(0, 2),
        reversed: false,
        parent_first_frame_origins: &[0.0, -40.0],
        parent_last_frame_origins: &[30.0, 80.0],
        current_first_frame_origins: &[0.0],
        current_last_frame_origins: &[30.0, 80.0],
        inherited: true,
    });
    assert_eq!(
        result,
        Err(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange),
    );
}

#[test]
fn grid_column_gap_separates_declared_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(210.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let expected = DefiniteTracks::new(210.0, 10.0)
        .track(Track::px(80.0))
        .track(Track::px(120.0))
        .solve();
    assert_eq!(output.size, Size::new(210.0, 40.0));
    assert_eq!(output.content_size, Size::new(210.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected.offset(0), 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(expected.offset(1), 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(expected.size(1), 40.0)
    );
}

#[test]
fn grid_item_margins_reduce_stretched_grid_area() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::px(3.0),
                right: LengthAuto::px(7.0),
                bottom: LengthAuto::px(5.0),
                left: LengthAuto::px(11.0),
            },
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let layout_input = tree
        .inputs(2)
        .iter()
        .find(|input| input.run_mode() == RunMode::PerformLayout)
        .expect("grid item should be laid out");
    assert_eq!(layout_input.known(), Size::new(Some(82.0), Some(32.0)));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(11.0, 3.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(82.0, 32.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.left,
        11.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.right,
        7.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.top,
        3.0
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").margin.bottom,
        5.0
    );
}

#[test]
fn grid_definite_column_line_places_item_in_explicit_track() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    for node in 2..=3 {
        tree.insert_children(node, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(2, 3).expect("valid grid lines"),
            grid_row: GridPlacement::try_line(1).expect("first grid row"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(3, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let columns = DefiniteTracks::new(200.0, 0.0)
        .track(Track::px(80.0))
        .track(Track::px(120.0))
        .solve();
    let column_area = LinePlacement::Lines { start: 2, end: 3 }
        .resolve_axis(1)
        .unwrap();
    let expected_column_area = columns.area(
        column_area.start_line as usize,
        column_area.end_line as usize,
    );

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected_column_area.start, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(expected_column_area.size, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
}

#[test]
fn grid_definite_row_line_places_item_in_explicit_track() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    for node in 2..=3 {
        tree.insert_children(node, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(60.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_row: GridPlacement::try_lines(2, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(3, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 20.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(80.0, 20.0)
    );
}

#[test]
fn grid_definite_column_span_covers_multiple_tracks_and_gap() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(210.0), PreferredSize::px(40.0)),
            grid_template_columns: vec![TrackComponent::px(80.0), TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(210.0, 40.0)
    );
}

#[test]
fn grid_definite_row_span_covers_multiple_tracks_and_gap() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::px(70.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(40.0)],
            gap: Size::new(Length::ZERO, Length::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 70.0)
    );
}

#[test]
fn grid_column_span_auto_places_across_multiple_free_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(150.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(40.0),
                TrackComponent::px(50.0),
                TrackComponent::px(60.0),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_span(2).expect("valid grid span"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(3, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let columns = DefiniteTracks::new(150.0, 0.0)
        .track(Track::px(40.0))
        .track(Track::px(50.0))
        .track(Track::px(60.0))
        .solve();
    let mut placement = AutoPlacer::try_new(3, 1, Flow::Row).unwrap();
    let first_area = placement.place(2, 1).unwrap();
    let second_area = placement.place(1, 1).unwrap();
    let expected_first_columns = columns.area(
        first_area.column_start,
        first_area.column_start + first_area.column_span,
    );
    let expected_second_columns = columns.area(
        second_area.column_start,
        second_area.column_start + second_area.column_span,
    );

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected_first_columns.start, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(expected_first_columns.size, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(expected_second_columns.start, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(expected_second_columns.size, 20.0)
    );
}

#[test]
fn grid_mixed_positive_negative_line_span_counts_actual_tracks_for_auto_growth() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::AUTO),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_lines(2, -1).expect("valid grid lines"),
                grid_row: GridPlacement::try_line(1).expect("first grid row"),
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default());

    compute_oracle_grid(&mut tree);
    let root = tree.final_layout(1).expect("root should be laid out");
    let spanning = tree
        .final_layout(2)
        .expect("spanning child should be laid out");
    let auto = tree.final_layout(3).expect("auto child should be laid out");

    assert_eq!(root.size.height, 20.0);
    assert_eq!(spanning.location, Point::new(40.0, 0.0));
    assert_eq!(spanning.size, Size::new(80.0, 20.0));
    assert_eq!(auto.location, Point::new(0.0, 0.0));
    assert_eq!(auto.size, Size::new(40.0, 20.0));
}

#[test]
fn grid_definite_column_end_line_resolves_to_previous_track() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(10.0)),
            grid_template_columns: vec![
                TrackComponent::px(20.0),
                TrackComponent::px(30.0),
                TrackComponent::px(40.0),
            ],
            grid_template_rows: vec![TrackComponent::px(10.0)],
            gap: Size::new(Length::px(5.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_end_line(3).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(25.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 10.0)
    );
}

#[test]
fn grid_definite_row_end_line_resolves_to_previous_track() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(20.0), PreferredSize::px(90.0)),
            grid_template_columns: vec![TrackComponent::px(20.0)],
            grid_template_rows: vec![
                TrackComponent::px(10.0),
                TrackComponent::px(20.0),
                TrackComponent::px(30.0),
            ],
            gap: Size::new(Length::ZERO, Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_row: GridPlacement::try_end_line(3).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(20.0, 20.0)
    );
}

#[test]
fn grid_justify_content_center_offsets_tracks_inside_inner_width() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(80.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            justify_content: Some(AlignContent::Center),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let expected = align_tracks_report(
        200.0,
        vec![80.0],
        0.0,
        TrackAlignment::Center,
        AlignmentSafety::Unsafe,
    );

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected.offsets[0], 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 20.0)
    );
}

#[test]
fn grid_justify_content_space_between_distributes_free_width_between_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(50.0), TrackComponent::px(50.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            justify_content: Some(AlignContent::SpaceBetween),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(150.0, 0.0)
    );
}

#[test]
fn grid_fraction_tracks_share_leftover_space_after_fixed_tracks_and_gaps() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(300.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![
                TrackComponent::px(50.0),
                track_component_flex(1.0),
                track_component_flex(2.0),
            ],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());
    tree.insert_style(4, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let expected = TrackSizingSlice::definite_columns(300.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::flex(1.0))
        .track(GridTrack::flex(2.0))
        .solve();
    assert_eq!(expected.final_tracks.len(), 3);
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected.final_tracks[0].offset, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(expected.final_tracks[0].size, 20.0)
    );
    assert!(
        (tree.layout(3).expect("node layout is staged").location.x
            - expected.final_tracks[1].offset)
            .abs()
            < 0.000_001
    );
    assert!(
        (tree.layout(3).expect("node layout is staged").size.width - expected.final_tracks[1].size)
            .abs()
            < 0.000_001
    );
    assert!(
        (tree.layout(4).expect("node layout is staged").location.x
            - expected.final_tracks[2].offset)
            .abs()
            < 0.000_001
    );
    assert!(
        (tree.layout(4).expect("node layout is staged").size.width - expected.final_tracks[2].size)
            .abs()
            < 0.000_001
    );
}

#[test]
fn grid_fraction_tracks_use_available_space_when_container_size_is_auto() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![track_component_flex(1.0), track_component_flex(2.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(12.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(200.0)),
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
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(120.0).unwrap()),
                        SizingOperation::Normal,
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(36.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(48.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(72.0, 20.0)
    );
}

#[test]
fn grid_fraction_tracks_clamp_available_space_to_min_size() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            min_size: Size::new(MinSize::px(180.0), MinSize::AUTO),
            grid_template_columns: vec![track_component_flex(1.0), track_component_flex(2.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(12.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(200.0)),
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
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(120.0).unwrap()),
                        SizingOperation::Normal,
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(56.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(68.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(112.0, 20.0)
    );
}

#[test]
fn grid_auto_fraction_tracks_resolve_after_required_tracks_are_known() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![TrackComponent::px(50.0)],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            grid_auto_columns: vec![track_component_flex(1.0)],
            gap: Size::new(Length::px(10.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(200.0), Some(100.0)),
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

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(140.0, 20.0)
    );
}

#[test]
fn grid_stretch_distributes_free_space_to_auto_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(220.0), PreferredSize::px(20.0)),
            grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::px(20.0)],
            gap: Size::new(Length::px(20.0), Length::ZERO),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());

    crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(220.0), Some(100.0)),
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

    let expected = TrackSizingSlice::definite_columns(220.0, 20.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .stretch_auto_tracks()
        .solve();

    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(expected.final_tracks[0].offset, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(expected.final_tracks[0].size, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(expected.final_tracks[1].offset, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(expected.final_tracks[1].size, 20.0)
    );
}

#[test]
fn grid_auto_track_uses_single_item_intrinsic_contribution() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 24.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: OracleGridArea::new(1, 1, 1, 1),
            min_content: 80.0,
            max_content: 80.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();
    let expected_rows = TrackSizingSlice::indefinite_rows(0.0)
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: OracleGridArea::new(1, 1, 1, 1),
            min_content: 24.0,
            max_content: 24.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    assert_eq!(
        output.size,
        Size::new(
            expected_columns.final_tracks[0].size,
            expected_rows.final_tracks[0].size
        )
    );
    assert_eq!(
        output.content_size,
        Size::new(
            expected_columns.final_tracks[0].size,
            expected_rows.final_tracks[0].size
        )
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(
            expected_columns.final_tracks[0].size,
            expected_rows.final_tracks[0].size
        )
    );
    assert_eq!(tree.inputs(2)[0].run_mode(), RunMode::ComputeSize);
    let layout_input = tree
        .inputs(2)
        .iter()
        .find(|input| input.run_mode() == RunMode::PerformLayout)
        .expect("grid item should be laid out after intrinsic measurement");
    assert_eq!(
        layout_input.known(),
        Size::new(
            Some(expected_columns.final_tracks[0].size),
            Some(expected_rows.final_tracks[0].size)
        )
    );
}

#[test]
fn grid_auto_width_does_not_stretch_auto_tracks_to_available_space() {
    let mut tree = OracleTree::new()
        .measure(2, ComputeOutput::from_outer_size(Size::new(80.0, 10.0)))
        .measure(3, ComputeOutput::from_outer_size(Size::new(80.0, 10.0)));
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(400.0), None),
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
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(400.0).unwrap()),
                        SizingOperation::Normal,
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

    assert_eq!(output.size, Size::new(160.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 10.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(80.0, 10.0)
    );
}

#[test]
fn grid_auto_width_ignores_loose_maximum_for_auto_track_stretch() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(0.0, 10.0)))
                .run_mode(RunMode::ComputeSize),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(100.0, 10.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            max_size: Size::new(MaxSize::px(260.0), MaxSize::NONE),
            grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::px(100.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(2).expect("valid grid line"),
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
            Size::new(None, None),
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

    // Max-content does not make free space definite. The unused Auto
    // column stays empty and the fixed column supplies the intrinsic width.
    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(100.0, 10.0)
    );
}

#[test]
fn grid_row_intrinsic_sizing_uses_resolved_column_width() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(30.0, 20.0)))
                .known(Size::new(Some(30.0), None))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
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
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
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
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(30.0, 20.0)))
                .known(Size::new(Some(30.0), Some(20.0)))
                .operations(
                    (Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(20.0).unwrap(),
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
                                crate::NonNegativeFiniteOf::new(30.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(20.0).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    ))
                    .map(|axis| {
                        (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                    }),
                ),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 10.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(30.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(output.size, Size::new(30.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(30.0, 20.0)
    );
    assert!(
        tree.inputs(2)
            .iter()
            .any(|input| input.run_mode() == RunMode::ComputeSize
                && input.known().width == Some(30.0)
                && input.sizing_request().axes().width
                    == AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(30.0).unwrap()),
                        SizingOperation::Normal
                    )),
        "grid row sizing should measure the item against the resolved column width"
    );
}

#[test]
fn grid_layout_percent_columns_rerun_row_sizing_with_resolved_width() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .known(Size::new(Some(80.0), None)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 96.0)))
                .known(Size::new(Some(80.0), Some(96.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(100.0, 64.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(80.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::percent(1.0)],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(output.size, Size::new(80.0, 96.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(80.0, 96.0)
    );
    assert!(
        tree.inputs(2)
            .iter()
            .any(|input| input.run_mode() == RunMode::ComputeSize
                && input.known().width == Some(80.0)
                && input.sizing_request().axes().width
                    == AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(80.0).unwrap()),
                        SizingOperation::Normal
                    )),
        "layout-time row sizing should be rerun against the resolved percent column width"
    );
}

#[test]
fn logical_ordinary_grid_carriers_project_fixed_tracks() {
    macro_rules! assert_projection {
        ($scalar:ty) => {
            for (writing_mode, direction) in [
                (WritingMode::HorizontalTb, Direction::Ltr),
                (WritingMode::HorizontalTb, Direction::Rtl),
                (WritingMode::VerticalRl, Direction::Ltr),
                (WritingMode::VerticalRl, Direction::Rtl),
                (WritingMode::VerticalLr, Direction::Ltr),
                (WritingMode::VerticalLr, Direction::Rtl),
                (WritingMode::SidewaysRl, Direction::Ltr),
                (WritingMode::SidewaysRl, Direction::Rtl),
                (WritingMode::SidewaysLr, Direction::Ltr),
                (WritingMode::SidewaysLr, Direction::Rtl),
            ] {
                let mut tree = OracleTreeOf::<$scalar>::new().children(1, []).style(
                    1,
                    NodeInputOf {
                        display: Display::Grid,
                        writing_mode,
                        direction,
                        grid_template_columns: vec![
                            TrackComponentOf::px(<$scalar as LayoutScalar>::from_f64(30.0)),
                            TrackComponentOf::px(<$scalar as LayoutScalar>::from_f64(40.0)),
                        ],
                        grid_template_rows: vec![
                            TrackComponentOf::px(<$scalar as LayoutScalar>::from_f64(50.0)),
                            TrackComponentOf::px(<$scalar as LayoutScalar>::from_f64(60.0)),
                        ],
                        ..NodeInputOf::default()
                    },
                );

                let output = crate::compute_grid(
                    &mut tree.borrow_compute(),
                    1,
                    ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        Size::NONE,
                        crate::ContainingLayoutContext::new(
                            crate::geometry::FlowAxes::new(
                                WritingMode::HorizontalTb,
                                Direction::Ltr,
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

                let expected = if writing_mode == WritingMode::HorizontalTb {
                    Size::new(
                        <$scalar as LayoutScalar>::from_f64(70.0),
                        <$scalar as LayoutScalar>::from_f64(110.0),
                    )
                } else {
                    Size::new(
                        <$scalar as LayoutScalar>::from_f64(110.0),
                        <$scalar as LayoutScalar>::from_f64(70.0),
                    )
                };
                assert_eq!(output.size, expected, "{writing_mode:?} {direction:?}");
            }
        };
    }

    assert_projection!(f32);
    assert_projection!(f64);
}

fn assert_logical_ordinary_grid_intrinsic_reruns_fake_measurements<S: LayoutScalar>()
where
    OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
{
    let scalar = S::from_f64;
    let measured_size = Size::new(scalar(17.0), scalar(31.0));
    let relationships = [
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "parallel",
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Rtl,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "opposing",
        ),
        (
            WritingMode::HorizontalTb,
            Direction::Ltr,
            WritingMode::VerticalRl,
            Direction::Ltr,
            "parent-horizontal-child-vertical",
        ),
        (
            WritingMode::VerticalLr,
            Direction::Ltr,
            WritingMode::HorizontalTb,
            Direction::Ltr,
            "parent-vertical-child-horizontal",
        ),
    ];

    for (parent_writing_mode, parent_direction, child_writing_mode, child_direction, label) in
        relationships
    {
        let mut tree = OracleTreeOf::<S>::new()
            .children(1, [2])
            .children(2, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    writing_mode: parent_writing_mode,
                    direction: parent_direction,
                    grid_template_columns: vec![TrackComponentOf::AUTO],
                    grid_template_rows: vec![TrackComponentOf::AUTO],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    writing_mode: child_writing_mode,
                    direction: child_direction,
                    ..NodeInputOf::default()
                },
            )
            .measure(2, ComputeOutputOf::from_outer_size(measured_size));

        let output = crate::compute_grid(
            &mut tree.borrow_compute(),
            1,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
        .expect("ordinary grid fake measurement layout succeeds");

        assert_eq!(output.size, measured_size, "{label}");
    }

    let mut tree = OracleTreeOf::<S>::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(80.0))),
                grid_template_columns: vec![TrackComponentOf::percent(scalar(1.0))],
                grid_template_rows: vec![TrackComponentOf::AUTO],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: WritingMode::HorizontalTb,
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            ComputeOutputOf::from_outer_size(Size::new(scalar(40.0), scalar(96.0))),
        );

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
    .expect("ordinary grid percentage rerun succeeds");

    assert_eq!(output.size, Size::new(scalar(40.0), scalar(80.0)));
    let child_inputs = tree.inputs(2);
    assert!(
        child_inputs.iter().any(|input| {
            input.run_mode() == RunMode::ComputeSize
                && input.known() == Size::new(None, Some(scalar(80.0)))
                && input.sizing_request().axes()
                    == Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(scalar(80.0)).unwrap(),
                            ),
                            SizingOperation::Normal,
                        ),
                    )
        }),
        "constrained row sizing must map the resolved logical inline track to the child's physical height: {child_inputs:?}"
    );
    // A stable MIN inventory does not authorize a column revisit. Verify the
    // actual final placement ingress retains both resolved physical dimensions.
    assert!(
        child_inputs.iter().any(|input| {
            input.run_mode() == RunMode::PerformLayout
                && input.known() == Size::new(Some(scalar(40.0)), Some(scalar(80.0)))
                && input.parent() == Size::new(Some(scalar(40.0)), Some(scalar(80.0)))
        }),
        "final physical dimensions lost: {child_inputs:?}"
    );
}

#[test]
fn logical_ordinary_grid_intrinsic_reruns_fake_measurements_f32() {
    assert_logical_ordinary_grid_intrinsic_reruns_fake_measurements::<f32>();
}

#[test]
fn logical_ordinary_grid_intrinsic_reruns_fake_measurements_f64() {
    assert_logical_ordinary_grid_intrinsic_reruns_fake_measurements::<f64>();
}

#[test]
fn grid_row_intrinsic_sizing_includes_item_vertical_margins() {
    let mut tree =
        OracleTree::new().measure(2, ComputeOutput::from_outer_size(Size::new(50.0, 10.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
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
            Size::new(Some(50.0), Some(100.0)),
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
                        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(50.0).unwrap()),
                        SizingOperation::Normal,
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

    assert_eq!(output.size, Size::new(50.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(50.0, 10.0)
    );
}

#[test]
fn grid_minmax_max_content_minimum_overrides_fixed_maximum() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(40.0, 40.0)))
                .known(Size::new(Some(40.0), Some(40.0))),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(40.0, 10.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::minmax(
                MinTrackSizing::MAX_CONTENT,
                MaxTrackSizing::px(10.0),
            )],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(output.size, Size::new(40.0, 40.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(40.0, 40.0)
    );
}

#[test]
fn grid_auto_placed_intrinsic_items_size_their_placed_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(10.0, 20.0)));
    tree.insert_measure(3, ComputeOutput::from_outer_size(Size::new(100.0, 20.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(output.size, Size::new(110.0, 20.0));
    assert_eq!(output.content_size, Size::new(110.0, 20.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(10.0, 20.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(10.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(100.0, 20.0)
    );
}

#[test]
fn grid_intrinsic_columns_exclude_cyclic_margins_then_resolve_used_margins() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(20.0, 10.0)))
                .run_mode(RunMode::ComputeSize),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(0.0, 10.0)));
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::percent(0.5),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::percent(0.5),
            },
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(20.0, 10.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    // The 20px auto track is the child's containing block. Each 50%
    // margin resolves to 10px after sizing and does not grow the track.
    assert_eq!(output.content_size.width, 200.0);
    let child = tree.layout(2).unwrap();
    assert_eq!((child.margin.left, child.margin.right), (10.0, 10.0));
}

#[test]
fn grid_nested_stretch_resolves_block_padding_percent_against_inline_size() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, crate::LayoutOutputEntryOf<u32, Scalar>>,
    }

    impl<'input> RecursiveTreeCompute<'input> {
        fn compute_node(
            &mut self,
            node: u32,
            input: ComputeInput,
        ) -> LayoutResultOf<u32, ComputeOutput, Scalar> {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return compute_leaf(input, &node_input, |measure_input| {
                    let known = measure_input.known_content_size();
                    Ok::<_, core::convert::Infallible>(Size::new(
                        known.width.unwrap_or(0.0),
                        known.height.unwrap_or(0.0),
                    ))
                })
                .map_err(|error| fake_leaf_error(node, error));
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
    }

    impl RecursiveTree {
        fn borrow_compute(&mut self) -> RecursiveTreeCompute<'_> {
            RecursiveTreeCompute {
                children: &self.children,
                styles: &self.styles,
                layouts: &mut self.layouts,
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
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![3]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            padding: Edges {
                top: Length::percent(0.2),
                right: Length::ZERO,
                bottom: Length::ZERO,
                left: Length::ZERO,
            },
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

    assert_eq!(output.size, Size::new(200.0, 40.0));
    assert_eq!(tree.layouts[&2].output().size, Size::new(200.0, 40.0));
    assert_eq!(tree.layouts[&3].output().size, Size::new(200.0, 40.0));
}

#[test]
fn grid_nested_percent_margins_resolve_against_resolved_nested_inline_size() {
    #[derive(Default)]
    struct RecursiveTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, crate::LayoutOutputEntryOf<u32, Scalar>>,
    }

    impl<'input> RecursiveTreeCompute<'input> {
        fn compute_node(
            &mut self,
            node: u32,
            input: ComputeInput,
        ) -> LayoutResultOf<u32, ComputeOutput, Scalar> {
            let node_input = self.styles[&node].clone();
            if self.children[&node].is_empty() {
                return compute_leaf(input, &node_input, |measure_input| {
                    let known = measure_input.known_content_size();
                    Ok::<_, core::convert::Infallible>(Size::new(
                        known.width.unwrap_or(0.0),
                        known.height.unwrap_or(0.0),
                    ))
                })
                .map_err(|error| fake_leaf_error(node, error));
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
    }

    impl RecursiveTree {
        fn borrow_compute(&mut self) -> RecursiveTreeCompute<'_> {
            RecursiveTreeCompute {
                children: &self.children,
                styles: &self.styles,
                layouts: &mut self.layouts,
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
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![3]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::percent(0.5), PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            size: Size::new(PreferredSize::percent(0.45), PreferredSize::AUTO),
            margin: Edges::all(LengthAuto::percent(0.05)),
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

    assert_eq!(output.size, Size::new(200.0, 10.0));
    assert_eq!(tree.layouts[&2].output().size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&3].output().location, Point::new(5.0, 5.0));
    assert_eq!(tree.layouts[&3].output().size, Size::new(45.0, 0.0));
}

#[test]
fn grid_spanning_item_grows_auto_track_after_min_content_track() {
    let mut tree = OracleTree::new()
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(40.0, 10.0)))
                .operations(Size::new(Some(SizingOperation::MinContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(80.0, 10.0)));
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::MIN_CONTENT, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());
    tree.insert_style(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
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

    assert_eq!(output.size, Size::new(80.0, 50.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(20.0, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(60.0, 40.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 40.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(80.0, 10.0)
    );
}

#[test]
fn grid_clipped_spanning_item_distributes_across_min_content_and_auto_tracks() {
    let mut tree = OracleTree::new()
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(40.0, 10.0)))
                .operations(Size::new(Some(SizingOperation::MinContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure(4, ComputeOutput::from_outer_size(Size::new(80.0, 10.0)));
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::MIN_CONTENT, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_style(3, NodeInput::default());
    tree.insert_style(
        4,
        NodeInput {
            overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
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

    // Limited min-content40 first gives20/20. Under the intrinsic MAX
    // constraint, only the Auto minimum receives the remaining40; the
    // min-content track keeps20. Both phases share the same occupancy.
    assert_eq!(output.size, Size::new(80.0, 50.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(20.0, 40.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(60.0, 40.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 40.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(80.0, 10.0)
    );
}

#[test]
fn grid_spanning_item_grows_underfilled_auto_track_first() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4]);
    for node in 2..=4 {
        tree.insert_children(node, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(320.0), PreferredSize::px(640.0)),
            grid_template_columns: vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                track_component_flex(1.0),
            ],
            grid_template_rows: vec![
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                TrackComponent::AUTO,
                track_component_flex(1.0),
            ],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(50.0)),
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        3,
        NodeInput {
            size: Size::new(PreferredSize::px(40.0), PreferredSize::px(30.0)),
            grid_column: GridPlacement::try_line_span(2, 2).expect("valid grid line span"),
            grid_row: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        4,
        NodeInput {
            size: Size::new(PreferredSize::px(120.0), PreferredSize::px(20.0)),
            grid_column: GridPlacement::try_line_span(1, 2).expect("valid grid line span"),
            grid_row: GridPlacement::try_line(3).expect("valid grid line"),
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

    assert_eq!(output.size, Size::new(320.0, 640.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(100.0, 50.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(100.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(40.0, 30.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(120.0, 20.0)
    );
}

#[test]
fn grid_spanning_item_reserves_percent_track_from_max_content_size() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(80.0, 40.0)))
                .operations(Size::new(Some(SizingOperation::MinContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(160.0, 40.0)));
    tree.insert_children(1, vec![2, 3, 4, 5, 6, 7, 8]);
    for node in 2..=8 {
        tree.insert_children(node, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::MIN_CONTENT,
                TrackComponent::MAX_CONTENT,
                TrackComponent::Track(crate::TrackSizing::fit_content(SizingCalculation::value(
                    lp(20.0, 0.0),
                ))),
                TrackComponent::AUTO,
                TrackComponent::px(10.0),
                TrackComponent::percent(0.2),
            ],
            grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 6).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    for node in 3..=8 {
        tree.insert_style(node, NodeInput::default());
    }

    // This performed request is normal layout in indefinite space. Intrinsic
    // item contributions choose the container; final percentage reservation
    // then uses its accepted extent rather than retaining formal MAX phases.
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
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
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
    assert!(tree.inputs(2).iter().any(|input| {
        input.sizing_request().axes()
            == Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(40.0).unwrap()),
                    SizingOperation::Normal,
                ),
            )
            && input.known().width.is_none()
    }));
    assert_eq!(output.size, Size::new(160.0, 80.0));
    let mut root_layout = NodeOutput::new();
    root_layout.size = output.size;
    root_layout.content_size = output.content_size;
    tree.borrow_compute()
        .set_unrounded(crate::test_support::layout_tree::ordinary_unrounded_box(
            1,
            root_layout,
        ));

    round_layout(&mut tree.borrow_compute(), 1).unwrap();
    assert_eq!(
        tree.final_layout(3)
            .expect("node final layout is staged")
            .size,
        Size::new(10.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(4)
            .expect("node final layout is staged")
            .location,
        Point::new(10.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(4)
            .expect("node final layout is staged")
            .size,
        Size::new(89.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(5)
            .expect("node final layout is staged")
            .location,
        Point::new(99.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(5)
            .expect("node final layout is staged")
            .size,
        Size::new(10.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(6)
            .expect("node final layout is staged")
            .location,
        Point::new(109.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(6)
            .expect("node final layout is staged")
            .size,
        Size::new(9.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(7)
            .expect("node final layout is staged")
            .location,
        Point::new(118.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(7)
            .expect("node final layout is staged")
            .size,
        Size::new(10.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(8)
            .expect("node final layout is staged")
            .location,
        Point::new(128.0, 40.0)
    );
    assert_eq!(
        tree.final_layout(8)
            .expect("node final layout is staged")
            .size,
        Size::new(32.0, 40.0)
    );
}

#[test]
fn grid_spanning_item_counts_definite_minmax_floors_when_reserving_percent_tracks() {
    let mut tree = OracleTree::new()
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_outer_size(Size::new(160.0, 40.0)))
                .operations(Size::new(Some(SizingOperation::MinContent), None))
                .spaces(Size::new(None, None)),
        )
        .measure(2, ComputeOutput::from_outer_size(Size::new(320.0, 40.0)));
    tree.insert_children(1, 2..=15);
    for node in 2..=15 {
        tree.insert_children(node, vec![]);
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::MIN_CONTENT,
                TrackComponent::MAX_CONTENT,
                TrackComponent::Track(crate::TrackSizing::fit_content(SizingCalculation::value(
                    lp(20.0, 0.0),
                ))),
                TrackComponent::AUTO,
                TrackComponent::px(10.0),
                TrackComponent::percent(0.2),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::AUTO),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::px(4.0)),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::MIN_CONTENT),
                TrackComponent::minmax(MinTrackSizing::px(2.0), MaxTrackSizing::MAX_CONTENT),
                TrackComponent::minmax(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::MAX_CONTENT),
                TrackComponent::minmax(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::AUTO),
                TrackComponent::minmax(MinTrackSizing::MAX_CONTENT, MaxTrackSizing::AUTO),
            ],
            grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line_span(1, 13).expect("valid grid line span"),
            ..NodeInput::default()
        },
    );
    for node in 3..=15 {
        tree.insert_style(node, NodeInput::default());
    }

    // This performed request is normal layout in indefinite space. Intrinsic
    // item contributions choose the container; final percentage reservation
    // then uses its accepted extent rather than retaining formal MAX phases.
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
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
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
    assert!(tree.inputs(2).iter().any(|input| {
        input.sizing_request().axes()
            == Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(40.0).unwrap()),
                    SizingOperation::Normal,
                ),
            )
            && input.known().width.is_none()
    }));
    assert!(tree.inputs(8).iter().any(|input| {
        input.run_mode() == RunMode::ComputeSize
            && input.sizing_request().axes().width
                == AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MinContent,
                )
            && input.known().width.is_none()
    }));
    let mut root_layout = NodeOutput::new();
    root_layout.size = output.size;
    root_layout.content_size = output.content_size;
    tree.borrow_compute()
        .set_unrounded(crate::test_support::layout_tree::ordinary_unrounded_box(
            1,
            root_layout,
        ));

    round_layout(&mut tree.borrow_compute(), 1).unwrap();
    let widths = (3..=15)
        .map(|node| {
            tree.final_layout(node)
                .expect("node final layout is staged")
                .size
                .width
        })
        .collect::<Vec<_>>();
    // The non-affected pass consumes the fixed minmax(2,4) headroom before
    // growing intrinsic tracks beyond their limits: total320, with no extra2
    // left for maximize. The stored browser322 remains separate corpus evidence.
    // Final percentages use accepted320; cumulative corner rounding applies to
    // the raw widths76/7, 80+76/7, 76/7, 76/7, 10, 64, 2, 4, 2, 2,
    // 76/7, 76/7, 80+76/7.
    assert_eq!(output.size, Size::new(320.0, 80.0));
    assert_eq!(
        widths,
        vec![
            11.0, 91.0, 11.0, 10.0, 10.0, 64.0, 2.0, 4.0, 2.0, 2.0, 11.0, 11.0, 91.0
        ]
    );
}

#[test]
fn grid_auto_size_re_resolves_indefinite_percentage_tracks_from_visible_content() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4]);
    for node in 2..=4 {
        tree.insert_children(node, vec![]);
        tree.insert_style(node, NodeInput::default());
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![
                TrackComponent::percent(0.4),
                TrackComponent::percent(0.4),
                TrackComponent::percent(0.4),
            ],
            grid_template_rows: vec![TrackComponent::percent(0.5), TrackComponent::percent(0.8)],
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_line(1).expect("valid grid line"),
            grid_row: GridPlacement::try_line(1).expect("valid grid line"),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
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
    tree.insert_style(
        4,
        NodeInput {
            grid_column: GridPlacement::try_line(3).expect("valid grid line"),
            grid_row: GridPlacement::try_line(2).expect("valid grid line"),
            ..NodeInput::default()
        },
    );

    // This is unbounded normal layout: choose the intrinsic container first,
    // then resolve its percentage tracks against that accepted size.
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
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
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

    assert_eq!(output.size, Size::new(100.0, 100.0));
    assert_eq!(
        tree.layout(3).expect("node layout is staged").location,
        Point::new(40.0, 0.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(40.0, 50.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(80.0, 50.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(40.0, 80.0)
    );
}

#[test]
fn grid_percent_rows_resolve_against_known_layout_height() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2, 3, 4, 5]);
    for node in 2..=5 {
        tree.insert_children(node, vec![]);
        tree.insert_style(node, NodeInput::default());
    }
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::px(20.0), TrackComponent::percent(0.1)],
            grid_template_rows: vec![TrackComponent::percent(0.3), TrackComponent::percent(0.1)],
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
            Size::new(Some(20.0), Some(10.0)),
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

    assert_eq!(output.size, Size::new(20.0, 10.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(20.0, 3.0)
    );
    assert_eq!(
        tree.layout(3).expect("node layout is staged").size,
        Size::new(2.0, 3.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").location,
        Point::new(0.0, 3.0)
    );
    assert_eq!(
        tree.layout(4).expect("node layout is staged").size,
        Size::new(20.0, 1.0)
    );
    assert_eq!(
        tree.layout(5).expect("node layout is staged").location,
        Point::new(20.0, 3.0)
    );
    assert_eq!(
        tree.layout(5).expect("node layout is staged").size,
        Size::new(2.0, 1.0)
    );
}

#[test]
fn grid_defaults_to_implicit_auto_tracks_when_no_auto_tracks_are_authored() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(2, NodeInput::default());
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(70.0, 18.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    assert_eq!(output.size, Size::new(70.0, 18.0));
    assert_eq!(output.content_size, Size::new(70.0, 18.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(70.0, 18.0)
    );
}

#[test]
fn grid_spanning_item_distributes_intrinsic_contribution_across_auto_tracks() {
    let mut tree = OracleTree::new();
    tree.insert_children(1, vec![2]);
    tree.insert_children(2, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
            grid_template_columns: vec![TrackComponent::AUTO, TrackComponent::AUTO],
            grid_template_rows: vec![TrackComponent::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInput::default()
        },
    );
    tree.insert_style(
        2,
        NodeInput {
            grid_column: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
            ..NodeInput::default()
        },
    );
    tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(100.0, 20.0)));

    let output = crate::compute_grid(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
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

    let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: OracleGridArea::new(1, 1, 2, 1),
            min_content: 100.0,
            max_content: 100.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();
    let expected_width = expected_columns
        .final_tracks
        .iter()
        .map(|track| track.size)
        .sum::<f32>();

    assert_eq!(output.content_size, Size::new(expected_width, 20.0));
    assert_eq!(
        tree.layout(2).expect("node layout is staged").location,
        Point::new(0.0, 0.0)
    );
    assert_eq!(
        tree.layout(2).expect("node layout is staged").size,
        Size::new(expected_width, 20.0)
    );
}

#[test]
fn grid_intrinsic_keyword_tracks_use_single_item_contribution() {
    fn run(track: TrackComponent) -> (ComputeOutput, NodeOutput) {
        let mut tree = OracleTree::new();
        tree.insert_children(1, vec![2]);
        tree.insert_children(2, vec![]);
        tree.insert_style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::AUTO, PreferredSize::AUTO),
                grid_template_columns: vec![track],
                grid_template_rows: vec![TrackComponent::AUTO],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInput::default()
            },
        );
        tree.insert_style(2, NodeInput::default());
        tree.insert_measure(2, ComputeOutput::from_outer_size(Size::new(90.0, 22.0)));

        let output = crate::compute_grid(
            &mut tree.borrow_compute(),
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(300.0), Some(200.0)),
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

        (output, tree.layout(2).expect("node layout is staged"))
    }

    for track in [TrackComponent::MIN_CONTENT, TrackComponent::MAX_CONTENT] {
        let (output, child_layout) = run(track);
        assert_eq!(output.content_size, Size::new(90.0, 22.0));
        assert_eq!(child_layout.location, Point::new(0.0, 0.0));
        assert_eq!(child_layout.size, Size::new(90.0, 22.0));
    }
}

#[test]
fn named_grid_spanning_item_counts_resolved_lines_for_auto_track_growth() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(40.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["b"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("a".to_string()),
                    RawGridLine::BareIdent("b".to_string()),
                ),
                ..NodeInput::DEFAULT
            },
        )
        .style(3, NodeInput::default());

    compute_oracle_grid(&mut tree);
    let spanning = tree
        .final_layout(2)
        .expect("spanning child should be laid out");
    let auto = tree.final_layout(3).expect("auto child should be laid out");

    assert_eq!(spanning.location, Point::new(0.0, 0.0));
    assert_eq!(spanning.size, Size::new(80.0, 20.0));
    assert_eq!(auto.location, Point::new(0.0, 20.0));
    assert_eq!(auto.size, Size::new(40.0, 20.0));
}

#[test]
fn fri08_c01_topology_empty_area_only_grid_uses_authored_auto_track_patterns() {
    let mut tree = OracleTree::new().children(1, []).style(
        1,
        NodeInput {
            display: Display::Grid,
            grid_auto_columns: vec![TrackComponent::px(40.0)],
            grid_auto_rows: vec![TrackComponent::px(20.0)],
            grid_template_areas: GridTemplateAreas {
                rows: vec![GridTemplateAreaRow {
                    cells: vec![
                        Some("main".to_string()),
                        Some("main".to_string()),
                        Some("main".to_string()),
                    ],
                }],
            },
            ..NodeInput::DEFAULT
        },
    );

    let output = compute_oracle_grid_output(&mut tree);

    assert_eq!(output.size, Size::new(120.0, 20.0));
}

#[test]
fn named_grid_negative_occurrence_and_missing_occurrence_extend_tracks() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(160.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_columns: vec![TrackComponent::px(40.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: -1,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedLine {
                        name: "a".to_string(),
                        index: 4,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let negative = tree
        .final_layout(2)
        .expect("negative occurrence child should be laid out");
    let missing = tree
        .final_layout(3)
        .expect("missing occurrence child should be laid out");

    assert_eq!(negative.location.x, 80.0);
    assert_eq!(negative.size.width, 40.0);
    assert_eq!(missing.location.x, 120.0);
    assert_eq!(missing.size.width, 40.0);
}

#[test]
fn named_grid_lone_named_span_auto_defaults_to_one_track_auto_placement() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(80.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                    TrackComponent::line_names(["a"]),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::NamedSpan {
                        name: "a".to_string(),
                        index: 2,
                    },
                    RawGridLine::Auto,
                ),
                ..NodeInput::DEFAULT
            },
        );

    compute_oracle_grid(&mut tree);
    let child = tree.final_layout(2).expect("child should be laid out");

    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 40.0);
}

#[test]
fn checked_track_phases_accept_f64_item_facts() {
    // The scalar-lane witness supplies equal minimum and min-content facts.
    let tracks = [
        TrackSizingOf::<f64>::px(10.25),
        TrackSizingOf::<f64>::AUTO,
        track_flex::<f64>(0.5),
    ];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, 1.5_f64, 1.5_f64, 4.5_f64),
            single_track_item(1, 2.5_f64, 2.5_f64, 5.5_f64),
            single_track_item(2, 3.5_f64, 3.5_f64, 6.5_f64),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: Some(90.75_f64),
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(90.75_f64),
                },
                gap: 0.25_f64,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(90.75_f64),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(90.75_f64),
            gap: 0.25_f64,
            alignment: AlignContent::Stretch,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    assert_eq!(sizes, vec![10.25_f64, 42.75_f64, 37.25_f64]);
}

#[test]
fn distribute_intrinsic_span_preserves_f64_fractional_shares() {
    let mut sizes = vec![1.25_f64, 3.75_f64];
    let tracks = [TrackSizingOf::<f64>::AUTO, TrackSizingOf::<f64>::AUTO];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        12.0_f64,
    );

    assert_eq!(sizes, vec![6.0_f64, 6.0_f64]);
}

#[test]
fn px_only_affine_max_track_keeps_its_min_content_minimum() {
    let tracks = [TrackSizing::new(
        MinTrackSizing::MinContent,
        MaxTrackSizing::Calculation(SizingCalculation::value(lp(24.0, 0.0))),
    )];

    let size = track_min_size_for_intrinsics(
        &tracks[0],
        None,
        GridTrackSizingConstraint {
            operation: SizingOperation::MaxContent,
            available_extent: None,
        },
        GridTrackContributions {
            minimum: 0.0,
            min_content: 11.0,
            max_content: 99.0,
        },
    );

    assert_eq!(size, 11.0);
}

#[test]
fn basis_dependent_affine_max_track_uses_max_content_base_fallback() {
    let tracks = [TrackSizing::new(
        MinTrackSizing::MinContent,
        MaxTrackSizing::Calculation(SizingCalculation::value(lp(0.0, 0.5))),
    )];

    let size = track_base_size_for_intrinsics(
        &tracks[0],
        None,
        GridTrackSizingConstraint {
            operation: SizingOperation::MaxContent,
            available_extent: None,
        },
        GridTrackContributions {
            minimum: 0.0,
            min_content: 11.0,
            max_content: 99.0,
        },
    );

    assert_eq!(size, 99.0);
}

#[test]
fn track_intrinsic_min_resolution_handles_invalid_affine_numeric_result() {
    let size = track_min_size_for_intrinsics(
        &TrackSizing::new(
            MinTrackSizing::Calculation(SizingCalculation::value(invalid_numeric_lp())),
            MaxTrackSizing::AUTO,
        ),
        Some(2.0),
        GridTrackSizingConstraint {
            operation: SizingOperation::MaxContent,
            available_extent: None,
        },
        GridTrackContributions {
            minimum: 11.0,
            min_content: 11.0,
            max_content: 99.0,
        },
    );

    assert_eq!(size, 0.0);
}

#[test]
fn track_intrinsic_max_resolution_handles_invalid_affine_numeric_result() {
    let size = track_base_size_for_intrinsics(
        &TrackSizing::new(
            MinTrackSizing::MinContent,
            MaxTrackSizing::Calculation(SizingCalculation::value(invalid_numeric_lp())),
        ),
        Some(2.0),
        GridTrackSizingConstraint {
            operation: SizingOperation::MaxContent,
            available_extent: None,
        },
        GridTrackContributions {
            minimum: 11.0,
            min_content: 11.0,
            max_content: 99.0,
        },
    );

    assert_eq!(size, 99.0);
}

#[test]
fn track_fit_content_limit_handles_invalid_affine_numeric_result() {
    let limit = track_growth_limit(
        &TrackSizing::new(
            MinTrackSizing::MinContent,
            MaxTrackSizing::FitContent(SizingCalculation::value(invalid_numeric_lp())),
        ),
        Some(2.0),
        GridTrackSizingConstraint {
            operation: SizingOperation::MaxContent,
            available_extent: None,
        },
        GridTrackContributions {
            minimum: 99.0,
            min_content: 99.0,
            max_content: 99.0,
        },
    );

    assert_eq!(limit, Some(99.0));
}

#[test]
fn named_grid_placement_context_ignores_non_in_flow_track_requirements() {
    let placements = vec![
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(100).expect("valid grid line"),
            row: GridPlacement::try_line(100).expect("valid grid line"),
            absolute_column: GridPlacement::try_line(100).expect("valid grid line"),
            absolute_row: GridPlacement::try_line(100).expect("valid grid line"),
            in_flow: false,
        },
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(-10).expect("valid grid line"),
            row: GridPlacement::AUTO,
            absolute_column: GridPlacement::try_line(-10).expect("valid grid line"),
            absolute_row: GridPlacement::AUTO,
            in_flow: false,
        },
        ResolvedGridItemPlacement {
            column: GridPlacement::try_line(2).expect("valid grid line"),
            row: GridPlacement::try_line(3).expect("valid grid line"),
            absolute_column: GridPlacement::try_line(2).expect("valid grid line"),
            absolute_row: GridPlacement::try_line(3).expect("valid grid line"),
            in_flow: true,
        },
    ];

    assert_eq!(
        lanes_axis_track_demand(&placements, 2, GridAxisKind::Column),
        Ok((0, 2))
    );
    assert_eq!(
        lanes_axis_track_demand(&placements, 2, GridAxisKind::Row),
        Ok((0, 3))
    );
}

#[test]
fn spanned_track_size_counts_tracks_and_internal_gaps() {
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 0, 1, 7.0), 20.0);
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 0, 2, 7.0), 67.0);
    assert_eq!(spanned_track_size(&[20.0, 40.0, 10.0], 1, 3, 7.0), 57.0);
}

#[test]
fn logical_grid_area_projection_maps_vertical_grid_tracks_without_collapsing_rows() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let containing_size = Size::new(110.0, 70.0);

    assert_eq!(
        flow_axes.physical_point(
            LogicalPointOf::new(0.0, 0.0),
            LogicalSizeOf::new(30.0, 50.0),
            containing_size,
        ),
        Point::new(60.0, 0.0)
    );
    assert_eq!(
        flow_axes.physical_point(
            LogicalPointOf::new(30.0, 50.0),
            LogicalSizeOf::new(40.0, 60.0),
            containing_size,
        ),
        Point::new(0.0, 30.0)
    );
}

#[test]
fn grid_item_sizing_transfers_min_block_through_aspect_ratio_to_inline_size() {
    let child_style = NodeInput {
        min_size: Size::new(MinSize::AUTO, MinSize::px(50.0)),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let sizing = grid_item_sizing_with_grid_flow_status(
        &child_style,
        &NodeInput::default(),
        Size::new(100.0, 100.0),
        Size::new(Some(100.0), Some(100.0)),
        FlowAxes::new(child_style.writing_mode, child_style.direction),
    )
    .unwrap();

    assert_eq!(sizing.known, Size::new(Some(200.0), Some(100.0)));
}

#[test]
fn grid_item_sizing_keeps_inline_stretch_when_min_inline_defines_aspect_ratio() {
    let child_style = NodeInput {
        min_size: Size::new(MinSize::px(50.0), MinSize::AUTO),
        aspect_ratio: AspectRatio::new(2.0),
        ..NodeInput::default()
    };

    let sizing = grid_item_sizing_with_grid_flow_status(
        &child_style,
        &NodeInput::default(),
        Size::new(100.0, 100.0),
        Size::new(Some(100.0), Some(100.0)),
        FlowAxes::new(child_style.writing_mode, child_style.direction),
    )
    .unwrap();

    assert_eq!(sizing.known, Size::new(Some(100.0), Some(50.0)));
}

#[test]
fn fr_span_contribution_distributes_by_flex_factor() {
    let tracks = [track_flex(1.0), track_flex(2.0)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
    );

    assert_eq!(sizes, [20.0, 40.0]);
}

#[test]
fn fr_span_contribution_subtracts_non_flex_base_tracks() {
    let tracks = [TrackSizing::MIN_CONTENT, track_flex(1.0)];
    let mut sizes = [10.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        40.0,
    );

    assert_eq!(sizes, [10.0, 30.0]);
}

#[test]
fn fr_span_contribution_normalizes_sub_one_factors() {
    let tracks = [track_flex(0.2), track_flex(0.3)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
    );

    assert_eq!(sizes, [24.0, 36.0]);
}

#[test]
fn fr_span_contribution_normalizes_sub_one_factors_after_non_flex_tracks() {
    let tracks = [TrackSizing::px(9.0), track_flex(0.5), track_flex(0.5)];
    let mut sizes = [0.0, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        18.0,
    );

    assert_eq!(sizes, [0.0, 4.5, 4.5]);
}

#[test]
fn fr_span_contribution_splits_zero_factors_evenly() {
    let tracks = [track_flex(0.0), track_flex(0.0)];
    let mut sizes = [0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        60.0,
    );

    assert_eq!(sizes, [30.0, 30.0]);
}

#[test]
fn fr_span_contribution_keeps_indefinite_percent_tracks_for_initial_sizing() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::fit_content(SizingCalculation::value(lp(20.0, 0.0))),
        TrackSizing::AUTO,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
        track_flex(1.0),
        track_flex(2.0),
    ];
    let mut sizes = [0.0; 8];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        160.0,
    );

    assert_eq!(sizes, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 50.0, 100.0]);
}

#[test]
fn fr_span_contribution_reserves_resolved_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::fit_content(SizingCalculation::value(lp(20.0, 0.0))),
        TrackSizing::AUTO,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
        track_flex(1.0),
        track_flex(2.0),
    ];
    let mut sizes = [0.0; 8];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        Some(160.0),
        160.0,
    );

    assert_eq!(sizes, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 39.333332, 78.666664]);
}

#[test]
fn max_content_span_prefers_max_content_track() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
    ];
    let mut sizes = [80.0, 80.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        320.0,
    );

    assert_eq!(sizes, [80.0, 230.0, 0.0]);
}

#[test]
fn max_content_span_prefers_max_content_track_over_min_content_maximum() {
    let tracks = [
        TrackSizing::MAX_CONTENT,
        TrackSizing::minmax(MinTrackSizing::MAX_CONTENT, MaxTrackSizing::MIN_CONTENT),
    ];
    let mut sizes = [40.0, 20.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        80.0,
    );

    assert_eq!(sizes, [60.0, 20.0]);
}

#[test]
fn min_content_span_counts_indefinite_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [0.0, 0.0, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MinContent {
            prioritize_min_tracks: false,
        },
        None,
        160.0,
    );

    assert_eq!(sizes, [42.666668, 42.666668, 0.0, 0.0]);
}

#[test]
fn max_content_span_keeps_indefinite_percent_tracks_for_initial_sizing() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [42.666668, 42.666668, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        None,
        320.0,
    );

    assert_eq!(sizes, [42.666668, 267.3333, 0.0, 0.0]);
}

#[test]
fn max_content_span_reserves_resolved_percent_tracks() {
    let tracks = [
        TrackSizing::MIN_CONTENT,
        TrackSizing::MAX_CONTENT,
        TrackSizing::px(10.0),
        TrackSizing::percent(0.2),
    ];
    let mut sizes = [42.666668, 42.666668, 0.0, 0.0];

    distribute_intrinsic_span(
        &mut sizes,
        &tracks,
        IntrinsicSpanContribution::MaxContent,
        Some(320.0),
        320.0,
    );

    assert_eq!(sizes, [42.666668, 203.33333, 0.0, 0.0]);
}

#[test]
fn indefinite_flex_tracks_preserve_two_authored_single_track_items() {
    // Two independent single-track items have20/40 for each intrinsic purpose.
    let tracks = [track_flex(1.0), track_flex(2.0)];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, 20.0, 20.0, 20.0),
            single_track_item(1, 40.0, 40.0, 40.0),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: None,
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    assert_eq!(sizes, [20.0, 40.0]);
}

#[test]
fn inline_sub_one_flex_tracks_keep_non_spanned_track_proportional_to_used_fraction() {
    let tracks = [track_flex(0.2), track_flex(0.3), track_flex(0.5)];
    let sizes = {
        // One actual60-wide item spans tracks1..3; track3 has a real zero item.
        let tracks = &tracks;
        let items = [
            GridIntrinsicContributionRow {
                span: GridTrackSpan::new(1, 3),
                contributions: GridTrackContributions {
                    minimum: 0.0,
                    min_content: 60.0,
                    max_content: 60.0,
                },
            },
            single_track_item(2, 0.0, 0.0, 0.0),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        // Grid1§11.5 distributes the unused sub-one proportion equally:
        // .2×60+15=27; .3×60+15=33. The zero item adds no floor.
        assert_eq!(phases.base_sizes, [27.0, 33.0, 0.0]);
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: None,
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    // Indefinite §11.7 uses max(27,33,0)=33 as the base-derived fr bound.
    // The spanning-item candidate freezes both smaller-flex tracks; the third
    // track grows to .5×33=16.5 while the first two retain their floors.
    assert_eq!(sizes, [27.0, 33.0, 16.5]);
}

// The same60-wide spanning item is resolved under a definite constraint here;
// its indefinite27/33 floors must not be imported into this separate phase.
fn assert_definite_partial_flex_leaves_host_space<S: LayoutScalar>() {
    let n = S::from_f64;
    let tracks = [track_flex(n(0.2)), track_flex(n(0.3))];
    let items = [GridIntrinsicContributionRow {
        span: GridTrackSpan::new(1, 3),
        contributions: GridTrackContributions {
            minimum: S::ZERO,
            min_content: n(60.0),
            max_content: n(60.0),
        },
    }];
    let phases = resolve_grid_contribution_rows(
        SpanningTrackContext {
            tracks: &tracks,
            basis: Some(n(200.0)),
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(n(200.0)),
            },
            gap: S::ZERO,
            gutters: None,
            base_floors: &[],
        },
        &items,
    )
    .expect("the actual minimum is zero in this definite phase");
    assert_eq!(phases.base_sizes, [S::ZERO; 2]);
    let sizes = resolve_tracks_from_phases(ResolvedAxisTrackInput {
        tracks: &tracks,
        phases: &phases,
        items: &items,
        available: GridTrackSizingConstraint {
            operation: SizingOperation::Normal,
            available_extent: Some(n(200.0)),
        },
        flex_bounds: GridTrackFinishingBounds::NONE,
        stretch_size: Some(n(200.0)),
        gap: S::ZERO,
        alignment: AlignContent::Start,
        gutters: None,
    })
    .expect("the definite fr search retains actual item incidence");
    // §11.7.1 clamps the .5 factor sum to1: fr=200. Preserve the
    // represented factors through each native-lane product and ordered sum;
    // the unused remainder is host space, not intrinsic content padding.
    let expected = [n(0.2) * n(200.0), n(0.3) * n(200.0)];
    let expected_sum = (S::ZERO + expected[0]) + expected[1];
    assert_eq!(sizes, expected);
    assert_eq!(track_sum(&sizes, S::ZERO), expected_sum);
    assert_eq!(
        n(200.0) - track_sum(&sizes, S::ZERO),
        n(200.0) - expected_sum
    );
}

#[test]
fn definite_partial_flex_leaves_host_space_in_both_scalar_lanes() {
    assert_definite_partial_flex_leaves_host_space::<f32>();
    assert_definite_partial_flex_leaves_host_space::<f64>();
}

#[test]
fn sub_one_flex_track_content_sum_uses_only_actual_track_extents() {
    // Actual indefinite sizes27/33/16.5 total76.5. An unused fractional
    // share is host space, not implicit content padding added by this sum.
    assert_eq!(track_sum(&[27.0, 33.0, 16.5], 0.0), 76.5);
}

#[test]
fn inline_minmax_tracks_shrink_to_minimum_bounds() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::minmax(MinTrackSizing::px(20.0), MaxTrackSizing::px(40.0)),
        TrackSizing::px(40.0),
    ];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, 0.0, 0.0, 0.0),
            single_track_item(1, 0.0, 0.0, 0.0),
            single_track_item(2, 0.0, 0.0, 0.0),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: Some(90.0),
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(90.0),
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(90.0),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(90.0),
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    assert_eq!(sizes, [40.0, 20.0, 40.0]);
}

#[test]
fn inline_minmax_tracks_grow_until_the_available_space_is_exhausted() {
    let tracks = [
        TrackSizing::px(40.0),
        TrackSizing::minmax(MinTrackSizing::px(20.0), MaxTrackSizing::px(40.0)),
        TrackSizing::px(40.0),
    ];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, 0.0, 0.0, 0.0),
            single_track_item(1, 0.0, 0.0, 0.0),
            single_track_item(2, 0.0, 0.0, 0.0),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: Some(110.0),
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(110.0),
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(110.0),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(110.0),
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    assert_eq!(sizes, [40.0, 30.0, 40.0]);
}

#[test]
fn inline_minmax_max_content_minimum_overrides_fixed_maximum() {
    let tracks = [TrackSizing::minmax(
        MinTrackSizing::MAX_CONTENT,
        MaxTrackSizing::px(10.0),
    )];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [single_track_item(0, 20.0, 20.0, 40.0)];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: None,
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    assert_eq!(sizes, [40.0]);
}

#[test]
fn inline_minmax_auto_minimum_allows_fixed_maximum() {
    let tracks = [TrackSizing::minmax(
        MinTrackSizing::AUTO,
        MaxTrackSizing::px(10.0),
    )];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [single_track_item(0, 10.0, 20.0, 40.0)];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: None,
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    assert_eq!(sizes, [10.0]);
}

#[test]
fn definite_flex_tracks_respect_larger_base_tracks() {
    // The middle fixed contribution is 100 for each intrinsic purpose.
    let tracks = [TrackSizing::px(40.0), track_flex(1.0), track_flex(1.0)];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, 0.0, 0.0, 0.0),
            single_track_item(1, 100.0, 100.0, 100.0),
            single_track_item(2, 0.0, 0.0, 0.0),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: Some(200.0),
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(200.0),
                },
                gap: 0.0,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(200.0),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(200.0),
            gap: 0.0,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };

    assert_eq!(sizes, [40.0, 100.0, 60.0]);
}

#[test]
fn grid_affine_percent_track_needs_layout_resolution() {
    let track = TrackSizing::new(
        MinTrackSizing::Calculation(SizingCalculation::value(lp(20.0, 0.10))),
        MaxTrackSizing::Calculation(SizingCalculation::value(lp(100.0, 0.0))),
    );

    assert!(track.depends_on_basis());
}

fn assert_grid_track_maximization<S: LayoutScalar>() {
    for row_axis in [false, true] {
        for (available, expected) in [(150.0, [75.0, 75.0]), (250.0, [100.0, 150.0])] {
            let tracks = vec![
                TrackComponentOf::minmax(
                    MinTrackSizingOf::px(S::ZERO),
                    MaxTrackSizingOf::px(S::from_f64(100.0)),
                ),
                TrackComponentOf::minmax(
                    MinTrackSizingOf::px(S::ZERO),
                    MaxTrackSizingOf::px(S::from_f64(200.0)),
                ),
            ];
            let cross_tracks = vec![TrackComponentOf::px(S::from_f64(10.0))];
            let size = if row_axis {
                Size::new(S::from_f64(10.0), S::from_f64(available))
            } else {
                Size::new(S::from_f64(available), S::from_f64(10.0))
            };
            let tree = PublicLayoutTreeOf::<S>::new().children(1, [2, 3]).style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    size: size.map(PreferredSizeOf::px),
                    grid_template_columns: if row_axis {
                        cross_tracks.clone()
                    } else {
                        tracks.clone()
                    },
                    grid_template_rows: if row_axis { tracks } else { cross_tracks },
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::<S>::default()
                },
            );
            let tree = tree
                .style(2, NodeInputOf::<S>::default())
                .style(3, NodeInputOf::<S>::default());
            let batch = fri08_c03_auto_fit_batch(&tree, size);
            for (node, expected) in [(2, expected[0]), (3, expected[1])] {
                let output = fri08_c01_placement_output(&batch, node);
                assert_eq!(
                    if row_axis {
                        output.size.height
                    } else {
                        output.size.width
                    },
                    S::from_f64(expected),
                    "axis row={row_axis}, available={available}"
                );
            }
        }
    }
}

#[test]
fn grid_track_maximization_equal_growth_and_frozen_limits_f32() {
    assert_grid_track_maximization::<f32>();
}

#[test]
fn grid_track_maximization_equal_growth_and_frozen_limits_f64() {
    assert_grid_track_maximization::<f64>();
}

fn assert_grid_axis_track_phase_contracts<S: LayoutScalar>() {
    let px = S::from_f64;
    let bounded = |min, max| {
        TrackSizingOf::minmax(MinTrackSizingOf::px(px(min)), MaxTrackSizingOf::px(px(max)))
    };
    let tracks = [bounded(20.0, 100.0), bounded(0.0, 200.0)];
    for (available, gap, expected) in [
        (
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(px(150.0)).unwrap()),
                SizingOperation::Normal,
            ),
            px(0.0),
            [85.0, 65.0],
        ),
        (
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(px(150.0)).unwrap()),
                SizingOperation::Normal,
            ),
            px(10.0),
            [80.0, 60.0],
        ),
        (
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(px(250.0)).unwrap()),
                SizingOperation::Normal,
            ),
            px(10.0),
            [100.0, 140.0],
        ),
        (
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
            px(10.0),
            [20.0, 0.0],
        ),
        (
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            px(10.0),
            [100.0, 200.0],
        ),
    ] {
        let sizes = {
            // Each listed row is one authored single-track item, including zero-sized items.
            let tracks = &tracks;
            let items = [
                single_track_item(0, S::ZERO, S::ZERO, S::ZERO),
                single_track_item(1, S::ZERO, S::ZERO, S::ZERO),
            ];
            let phases = resolve_grid_contribution_rows(
                SpanningTrackContext {
                    tracks,
                    basis: None,
                    available: GridTrackSizingConstraint {
                        operation: (available).operation(),
                        available_extent: (available)
                            .space()
                            .definite_size()
                            .map(crate::NonNegativeFiniteOf::get),
                    },
                    gap,
                    gutters: None,
                    base_floors: &[],
                },
                &items,
            )
            .expect("authored item incidence resolves checked phases");
            resolve_tracks_from_phases(ResolvedAxisTrackInput {
                tracks,
                phases: &phases,
                items: &items,
                available: GridTrackSizingConstraint {
                    operation: (available).operation(),
                    available_extent: (available)
                        .space()
                        .definite_size()
                        .map(crate::NonNegativeFiniteOf::get),
                },
                flex_bounds: GridTrackFinishingBounds::NONE,
                stretch_size: None,
                gap,
                alignment: AlignContent::Start,
                gutters: None,
            })
            .expect("authored track phases finish without losing item spans")
        };
        assert_eq!(
            sizes,
            expected.map(px),
            "unequal bases under {available:?} and gap {gap:?}"
        );
    }
    let tracks = [
        bounded(0.0, 100.0),
        TrackSizingOf::px(px(80.0)),
        bounded(0.0, 200.0),
    ];
    let gutters = OrdinaryGridAxisGuttersOf::new(3, &[false, true, false], px(10.0));
    let sizes = {
        // Only active tracks0/2 contain real zero-sized items; track1 is collapsed.
        let tracks = &tracks;
        let items = [
            single_track_item(0, S::ZERO, S::ZERO, S::ZERO),
            single_track_item(2, S::ZERO, S::ZERO, S::ZERO),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(px(150.0)),
                },
                gap: px(10.0),
                gutters: Some(&gutters),
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(px(150.0)),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(px(150.0)),
            gap: px(10.0),
            alignment: AlignContent::Start,
            gutters: Some(&gutters),
        })
        .expect("authored track phases finish without losing item spans")
    };
    assert_eq!(
        sizes,
        [px(70.0), S::ZERO, px(70.0)],
        "collapsed tracks do not grow or introduce additional gutters"
    );

    let tracks = [
        TrackSizingOf::px(px(40.0)),
        bounded(0.0, 100.0),
        track_flex(S::ONE),
        TrackSizingOf::AUTO,
    ];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, S::ZERO, S::ZERO, S::ZERO),
            single_track_item(1, S::ZERO, S::ZERO, S::ZERO),
            single_track_item(2, S::ZERO, S::ZERO, S::ZERO),
            single_track_item(3, px(20.0), px(20.0), px(30.0)),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(px(300.0)),
                },
                gap: S::ZERO,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(px(300.0)),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(px(300.0)),
            gap: S::ZERO,
            alignment: AlignContent::Stretch,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };
    assert_eq!(
        sizes,
        [40.0, 100.0, 130.0, 30.0].map(px),
        "maximize before resolving fr and stretching auto tracks"
    );
    let tracks = [
        TrackSizingOf::px(px(40.0)),
        bounded(0.0, 100.0),
        TrackSizingOf::AUTO,
    ];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, S::ZERO, S::ZERO, S::ZERO),
            single_track_item(1, S::ZERO, S::ZERO, S::ZERO),
            single_track_item(2, px(20.0), px(20.0), px(30.0)),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(px(300.0)),
                },
                gap: S::ZERO,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(px(300.0)),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(px(300.0)),
            gap: S::ZERO,
            alignment: AlignContent::Stretch,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };
    assert_eq!(
        sizes,
        [40.0, 100.0, 160.0].map(px),
        "only auto maxima stretch after maximization"
    );

    let tracks = [TrackSizingOf::percent(px(0.5)), bounded(0.0, 200.0)];
    let sizes = {
        // Each listed row is one authored single-track item, including zero-sized items.
        let tracks = &tracks;
        let items = [
            single_track_item(0, S::ZERO, S::ZERO, S::ZERO),
            single_track_item(1, S::ZERO, S::ZERO, S::ZERO),
        ];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: Some(px(200.0)),
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(px(150.0)),
                },
                gap: S::ZERO,
                gutters: None,
                base_floors: &[],
            },
            &items,
        )
        .expect("authored item incidence resolves checked phases");
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &items,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(px(150.0)),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: Some(px(150.0)),
            gap: S::ZERO,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .expect("authored track phases finish without losing item spans")
    };
    assert_eq!(
        sizes,
        [100.0, 50.0].map(px),
        "percentage basis is independent from space for track growth"
    );
}

#[test]
fn grid_axis_track_phase_contracts_f32() {
    assert_grid_axis_track_phase_contracts::<f32>();
}

#[test]
fn grid_axis_track_phase_contracts_f64() {
    assert_grid_axis_track_phase_contracts::<f64>();
}

fn assert_size_containment_grid_controls<S: LayoutScalar>() {
    let px = S::from_f64;
    for (children, size_containment, expected_width) in [
        (vec![], crate::SizeContainment::None, 10.0),
        (vec![3], crate::SizeContainment::None, 70.0),
        (vec![3], crate::SizeContainment::BothAxes, 10.0),
    ] {
        let has_content = !children.is_empty();
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .children(2, children)
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(px(20.0))),
                    grid_template_columns: vec![TrackComponentOf::AUTO],
                    grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(px(20.0))),
                    grid_template_columns: vec![
                        TrackComponentOf::px(px(10.0)),
                        TrackComponentOf::AUTO,
                    ],
                    grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                    justify_content: Some(AlignContent::Start),
                    size_containment,
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(px(60.0)), PreferredSizeOf::px(px(20.0))),
                    grid_column: GridPlacement::try_line(2).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch = fri08_c03_auto_fit_batch(&tree, Size::new(px(200.0), px(20.0)));
        for node in [1, 2] {
            assert_eq!(
                fri08_c01_placement_output(&batch, node).size.width,
                px(expected_width)
            );
        }
        if has_content {
            let child = fri08_c01_placement_output(&batch, 3);
            assert_eq!((child.location.x, child.size.width), (px(10.0), px(60.0)));
        }
    }
}

#[test]
fn size_containment_grid_controls() {
    assert_size_containment_grid_controls::<f32>();
    assert_size_containment_grid_controls::<f64>();
}

fn assert_size_containment_subgrid<S: LayoutScalar>() {
    let px = S::from_f64;
    for (children, size_containment, expected_width) in [
        (vec![], crate::SizeContainment::None, 15.0),
        (vec![3], crate::SizeContainment::None, 75.0),
        (vec![3], crate::SizeContainment::InlineAxis, 15.0),
        (vec![3], crate::SizeContainment::BothAxes, 15.0),
    ] {
        let has_content = !children.is_empty();
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .children(2, children)
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(px(20.0))),
                    grid_template_columns: vec![
                        TrackComponentOf::px(px(10.0)),
                        TrackComponentOf::AUTO,
                    ],
                    grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_columns: subgrid_track_of(),
                    grid_column: GridPlacement::try_span(2).unwrap(),
                    padding: Edges::new(
                        LengthOf::ZERO,
                        LengthOf::px(px(5.0)),
                        LengthOf::ZERO,
                        LengthOf::ZERO,
                    ),
                    size_containment,
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::px(px(60.0)), PreferredSizeOf::px(px(20.0))),
                    grid_column: GridPlacement::try_line(2).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch = fri08_c03_auto_fit_batch(&tree, Size::new(px(200.0), px(20.0)));
        for node in [1, 2] {
            assert_eq!(
                fri08_c01_placement_output(&batch, node).size.width,
                px(expected_width)
            );
        }
        if has_content {
            let child = fri08_c01_placement_output(&batch, 3);
            assert_eq!((child.location.x, child.size.width), (px(10.0), px(60.0)));
        }
    }
}

#[test]
fn size_containment_preserves_subgrid_inheritance() {
    assert_size_containment_subgrid::<f32>();
    assert_size_containment_subgrid::<f64>();
}

fn assert_size_containment_grid_repeat_basis<S: LayoutScalar>() {
    let px = S::from_f64;
    let repeat =
        crate::TrackRepetitionOf::auto_fit_components(vec![TrackComponentOf::px(px(20.0))])
            .unwrap();
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(px(20.0))),
                max_size: Size::new(crate::MaxSizeOf::px(px(100.0)), crate::MaxSizeOf::NONE),
                grid_template_columns: vec![
                    TrackComponentOf::px(px(40.0)),
                    TrackComponentOf::Repeat(repeat),
                ],
                grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                grid_auto_columns: vec![TrackComponentOf::px(px(7.0))],
                justify_content: Some(AlignContent::Start),
                size_containment: crate::SizeContainment::BothAxes,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                grid_column: GridPlacement::try_line(4).unwrap(),
                ..NodeInputOf::default()
            },
        );
    let batch = fri08_c03_auto_fit_batch(&tree, Size::new(px(100.0), px(20.0)));
    assert_eq!(fri08_c01_placement_output(&batch, 1).size.width, px(40.0));
    let child = fri08_c01_placement_output(&batch, 2);
    assert_eq!((child.location.x, child.size.width), (px(40.0), px(20.0)));
}

#[test]
fn size_containment_preserves_grid_auto_fit_basis() {
    assert_size_containment_grid_repeat_basis::<f32>();
    assert_size_containment_grid_repeat_basis::<f64>();
}

fn assert_size_containment_grid_final_bases<S: LayoutScalar>() {
    let px = S::from_f64;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(px(20.0))),
                grid_template_columns: vec![
                    TrackComponentOf::px(px(40.0)),
                    TrackComponentOf::percent(px(0.5)),
                ],
                grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                justify_content: Some(AlignContent::Start),
                size_containment: crate::SizeContainment::BothAxes,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::percent(px(0.5)),
                    PreferredSizeOf::px(px(20.0)),
                ),
                grid_column: GridPlacement::try_line(2).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::percent(px(0.5)),
                    PreferredSizeOf::px(px(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    let batch = fri08_c03_auto_fit_batch(&tree, Size::new(px(200.0), px(20.0)));
    assert_eq!(fri08_c01_placement_output(&batch, 1).size.width, px(40.0));
    let item = fri08_c01_placement_output(&batch, 2);
    assert_eq!((item.location.x, item.size.width), (px(40.0), px(10.0)));
    // The final 40-unit content box resolves the percentage track to 20.
    // Established tracks therefore overflow to 60, which is the absolute
    // Auto/Auto containing area; its 50% child uses 30 rather than 20.
    let overflow = fri08_c01_placement_output(&batch, 1)
        .scroll_geometry
        .unwrap()
        .scrollable_overflow();
    assert_eq!(overflow.origin(), Point::ZERO);
    assert_eq!(overflow.size(), Size::new(px(60.0), px(20.0)));
    assert_eq!(fri08_c01_placement_output(&batch, 3).size.width, px(30.0));
}

#[test]
fn size_containment_preserves_grid_final_percentage_bases() {
    assert_size_containment_grid_final_bases::<f32>();
    assert_size_containment_grid_final_bases::<f64>();
}

fn assert_grid_minimum_finishes_intrinsic_tracks<S: LayoutScalar>() {
    let px = S::from_f64;
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let request = crate::LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        flow,
        Size::NONE,
    );
    let flex = TrackComponentOf::minmax(
        MinTrackSizingOf::px(S::ZERO),
        MaxTrackSizingOf::flex(TrackFlexFactorOf::try_new(px(0.5)).unwrap()),
    );
    let mut failures = Vec::new();
    for (case, tracks, align, minimum_widths) in [
        ("flex", vec![flex.clone()], AlignContent::Start, &[45.0][..]),
        (
            "auto",
            vec![TrackComponentOf::AUTO],
            AlignContent::Stretch,
            &[90.0][..],
        ),
        (
            "flex and auto",
            vec![flex, TrackComponentOf::AUTO],
            AlignContent::Stretch,
            &[45.0, 45.0][..],
        ),
    ] {
        for (minimum, maximum) in [(false, false), (false, true), (true, false), (true, true)] {
            let mut columns = vec![TrackComponentOf::px(px(10.0))];
            columns.extend(tracks.iter().cloned());
            let child_count = u32::try_from(tracks.len()).unwrap();
            let mut tree = PublicLayoutTreeOf::<S>::new()
                .children(1, 2..2 + child_count)
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Grid,
                        size: Size::new(
                            PreferredSizeOf::MAX_CONTENT,
                            PreferredSizeOf::px(px(20.0)),
                        ),
                        min_size: Size::new(
                            if minimum {
                                MinSizeOf::px(px(100.0))
                            } else {
                                MinSizeOf::AUTO
                            },
                            MinSizeOf::AUTO,
                        ),
                        max_size: Size::new(
                            if maximum {
                                crate::MaxSizeOf::px(px(200.0))
                            } else {
                                crate::MaxSizeOf::NONE
                            },
                            crate::MaxSizeOf::NONE,
                        ),
                        grid_template_columns: columns,
                        grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                        justify_content: Some(align),
                        ..NodeInputOf::default()
                    },
                );
            for node in 2..2 + child_count {
                tree = tree.style(
                    node,
                    NodeInputOf {
                        grid_column: GridPlacement::try_line(isize::try_from(node).unwrap())
                            .unwrap(),
                        ..NodeInputOf::default()
                    },
                );
            }
            let batch = compute_layout(&tree, 1, request).unwrap();
            let root = fri08_c01_placement_output(&batch, 1);
            let actual: Vec<_> = (2..2 + child_count)
                .map(|node| {
                    let marker = fri08_c01_placement_output(&batch, node);
                    (marker.location.x, marker.size.width)
                })
                .collect();
            // The minimum supplies 90px after the fixed track. Section 12.7
            // gives 0.5fr half that space, then §12.8 stretches Auto into any
            // remaining space. A loose maximum supplies no free space.
            let mut x = 10.0;
            let expected: Vec<_> = minimum_widths
                .iter()
                .map(|width| {
                    let width = if minimum { *width } else { 0.0 };
                    let geometry = (px(x), px(width));
                    x += width;
                    geometry
                })
                .collect();
            let outer = px(if minimum { 100.0 } else { 10.0 });
            if root.size.width != outer || actual != expected {
                failures.push(format!(
                    "{case} minimum={minimum} maximum={maximum}, expected {outer:?} {expected:?}, got {:?} {actual:?}",
                    root.size.width,
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn grid_minimum_finishes_intrinsic_tracks_f32() {
    assert_grid_minimum_finishes_intrinsic_tracks::<f32>();
}

#[test]
fn grid_minimum_finishes_intrinsic_tracks_f64() {
    assert_grid_minimum_finishes_intrinsic_tracks::<f64>();
}

fn assert_grid_intrinsic_track_constraints<S: LayoutScalar>() {
    for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalRl] {
        let flow = FlowAxes::new(writing_mode, Direction::Ltr);
        for row_axis in [false, true] {
            let mut cases = vec![
                ("min-content", PreferredSizeOf::MIN_CONTENT, [0.0, 0.0]),
                ("max-content", PreferredSizeOf::MAX_CONTENT, [100.0, 200.0]),
            ];
            if row_axis {
                cases.push(("auto block size", PreferredSizeOf::AUTO, [100.0, 200.0]));
            }
            for (case, preferred, expected) in cases {
                let tracks = vec![
                    TrackComponentOf::minmax(
                        MinTrackSizingOf::px(S::ZERO),
                        MaxTrackSizingOf::px(S::from_f64(100.0)),
                    ),
                    TrackComponentOf::minmax(
                        MinTrackSizingOf::px(S::ZERO),
                        MaxTrackSizingOf::px(S::from_f64(200.0)),
                    ),
                ];
                let cross = vec![TrackComponentOf::px(S::from_f64(10.0))];
                let size = if row_axis {
                    LogicalSizeOf::new(PreferredSizeOf::px(S::from_f64(10.0)), preferred)
                } else {
                    LogicalSizeOf::new(preferred, PreferredSizeOf::px(S::from_f64(10.0)))
                };
                let tree = PublicLayoutTreeOf::<S>::new()
                    .children(1, [2, 3])
                    .style(
                        1,
                        NodeInputOf {
                            display: Display::Grid,
                            writing_mode,
                            size: flow.physical_size(size),
                            grid_template_columns: if row_axis {
                                cross.clone()
                            } else {
                                tracks.clone()
                            },
                            grid_template_rows: if row_axis { tracks } else { cross },
                            justify_content: Some(AlignContent::Start),
                            align_content: Some(AlignContent::Start),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        2,
                        NodeInputOf {
                            writing_mode,
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            writing_mode,
                            ..NodeInputOf::default()
                        },
                    );
                let batch = fri08_c03_auto_fit_batch(&tree, Size::splat(S::from_f64(240.0)));
                for (node, expected) in [
                    (1, expected[0] + expected[1]),
                    (2, expected[0]),
                    (3, expected[1]),
                ] {
                    let size = flow.logical_size(fri08_c01_placement_output(&batch, node).size);
                    assert_eq!(
                        if row_axis { size.block } else { size.inline },
                        S::from_f64(expected),
                        "{case}, row={row_axis}, {writing_mode:?}, node {node}"
                    );
                }
            }
        }
    }
}

#[test]
fn grid_intrinsic_track_constraints_override_finite_viewport_f32() {
    assert_grid_intrinsic_track_constraints::<f32>();
}

#[test]
fn grid_intrinsic_track_constraints_override_finite_viewport_f64() {
    assert_grid_intrinsic_track_constraints::<f64>();
}

fn assert_intrinsic_traversal_allocation_authority<S: LayoutScalar>() {
    use crate::grid::subgrid::{
        GridMinimumParentFunctions, SubgridParentTrackSource, SubgridTraversalError,
        intrinsic_traversal_area_size,
    };
    let n = S::from_f64;
    let rows = [n(7.0)];
    let row_geometry = UsedGridAxisGeometryOf::new(rows.to_vec(), vec![false], S::ZERO);
    let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let area = GridArea {
        column: 0,
        column_end: 1,
        row: 0,
        row_end: 1,
        // The preliminary area estimate does not establish used track size.
        size: LogicalSizeOf::new(n(58.0), n(7.0)),
    };
    for (tracks, allocated, container, expected) in [
        (vec![n(86.0)], true, Some(n(58.0)), n(86.0)),
        (vec![n(86.0)], false, Some(n(58.0)), n(58.0)),
        (vec![S::ZERO], true, Some(n(58.0)), S::ZERO),
        (vec![S::ZERO], false, Some(n(58.0)), n(58.0)),
        (vec![n(86.0)], false, None, n(86.0)),
        (vec![S::ZERO], false, None, S::ZERO),
        (vec![], false, Some(n(58.0)), n(58.0)),
        (vec![], false, None, S::ZERO),
    ] {
        let column_geometry =
            UsedGridAxisGeometryOf::new(tracks.clone(), vec![false; tracks.len()], S::ZERO);
        let source = SubgridParentTrackSource {
            baseline_references: crate::GridBaselineReferences::default(),
            flow_axes: axes,
            gap: Size::ZERO,
            columns: Some(&column_geometry),
            rows: Some(&row_geometry),
            minimum_functions: GridMinimumParentFunctions {
                columns: None,
                rows: None,
            },
            algorithm: None,
            lane_grid_axis: None,
            allocated: LogicalSizeOf::new(allocated.then_some(tracks.as_slice()), Some(&rows)),
        };
        let actual = intrinsic_traversal_area_size(
            area,
            &tracks,
            &rows,
            Size::new(container, Some(n(99.0))),
            source,
        )
        .unwrap();
        assert_eq!(actual, Size::new(expected, n(7.0)));
        assert_eq!(
            axes.physical_size(LogicalSizeOf::new(actual.width, actual.height)),
            Size::new(n(7.0), expected),
            "allocated columns/rows remain in the owning logical axes",
        );
    }

    // The real span includes its actual gutter, including when it is the
    // whole grid and exceeds the containing content extent58.
    let tracks = [n(30.0), n(46.0)];
    let geometry = UsedGridAxisGeometryOf::new(tracks.to_vec(), vec![false; 2], n(10.0));
    let source = SubgridParentTrackSource {
        baseline_references: crate::GridBaselineReferences::default(),
        flow_axes: axes,
        gap: Size::new(n(10.0), S::ZERO),
        columns: Some(&geometry),
        rows: Some(&row_geometry),
        minimum_functions: GridMinimumParentFunctions {
            columns: None,
            rows: None,
        },
        algorithm: None,
        lane_grid_axis: None,
        allocated: LogicalSizeOf::new(Some(&tracks), Some(&rows)),
    };
    let full = GridArea {
        column_end: 2,
        ..area
    };
    let container = Size::new(Some(n(58.0)), Some(n(99.0)));
    assert_eq!(
        intrinsic_traversal_area_size(full, &tracks, &rows, container, source).unwrap(),
        Size::new(n(86.0), n(7.0)),
    );
    assert_eq!(
        intrinsic_traversal_area_size(area, &tracks, &rows, container, source).unwrap(),
        Size::new(n(30.0), n(7.0)),
    );
    let provisional = SubgridParentTrackSource {
        baseline_references: crate::GridBaselineReferences::default(),
        allocated: LogicalSizeOf::new(None, Some(&rows)),
        ..source
    };
    assert_eq!(
        intrinsic_traversal_area_size(full, &tracks, &rows, container, provisional).unwrap(),
        Size::new(n(58.0), n(7.0)),
    );
    let invalid = GridArea {
        column_end: 3,
        ..area
    };
    assert_eq!(
        intrinsic_traversal_area_size(invalid, &tracks, &rows, container, source),
        Err(SubgridTraversalError::SpanOutOfRange),
    );
}

#[test]
fn intrinsic_traversal_allocation_authority_f32() {
    assert_intrinsic_traversal_allocation_authority::<f32>();
}

#[test]
fn intrinsic_traversal_allocation_authority_f64() {
    assert_intrinsic_traversal_allocation_authority::<f64>();
}

#[path = "tracks_intrinsic/native_flex_spans.rs"]
mod native_flex_spans;

#[path = "tracks_intrinsic/native_flex_budget.rs"]
mod native_flex_budget;

#[path = "tracks_intrinsic/native_minmax_caps.rs"]
mod native_minmax_caps;

#[path = "tracks_intrinsic/native_flex_three_tracks.rs"]
mod native_flex_three_tracks;

#[path = "tracks_intrinsic/native_cyclic_subgrid_gap.rs"]
mod native_cyclic_subgrid_gap;

#[path = "tracks_intrinsic/native_thirteen_tracks.rs"]
mod native_thirteen_tracks;

#[path = "tracks_intrinsic/native_min_fit_content.rs"]
mod native_min_fit_content;

#[path = "tracks_intrinsic/native_fit_content_caps.rs"]
mod native_fit_content_caps;

fn assert_cyclic_gap_preserves_fixed_term<S: LayoutScalar>(display: Display) {
    let px = S::from_f64;
    let width = NonNegativeFiniteOf::new(px(40.0)).unwrap();
    let viewport = Size::new(Some(width), None);
    let frame = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        viewport,
        OrthogonalFallbackOf::new(
            viewport,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let request = |operation| {
        frame.request(Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, operation),
        ))
    };
    for (fixed, fraction, definite, root_height, second_start) in [
        (4.0, 0.0, false, 24.0, 14.0),
        (0.0, 0.25, false, 20.0, 15.0),
        (4.0, 0.25, true, 40.0, 24.0),
        (4.0, 0.25, false, 24.0, 20.0),
        (-4.0, 0.25, false, 20.0, 11.0),
        (-8.0, 0.25, false, 20.0, 10.0),
    ] {
        let mut root = NodeInputOf {
            display,
            size: Size::new(
                PreferredSizeOf::px(width.get()),
                if definite {
                    PreferredSizeOf::px(px(40.0))
                } else {
                    PreferredSizeOf::AUTO
                },
            ),
            grid_template_rows: vec![TrackComponentOf::px(px(10.0)); 2],
            gap: Size::new(
                LengthOf::ZERO,
                LengthOf::value(
                    LengthPercentageOf::from_coefficients(px(fixed), px(fraction)).unwrap(),
                ),
            ),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            justify_items: Some(AlignItems::Start),
            align_items: Some(AlignItems::Start),
            ..NodeInputOf::default()
        };
        if display == Display::Grid {
            root.grid_template_columns = vec![TrackComponentOf::px(px(20.0))];
        }
        let child = |row| NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(px(10.0))),
            grid_row: GridPlacement::try_line(row).unwrap(),
            ..NodeInputOf::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(0, root)
            .style(1, child(1))
            .style(2, child(2));
        let normal = compute_layout(&tree, 0, request(SizingOperation::Normal)).unwrap();
        let output = |node| readback::single_box_output(&normal, node, Phase::Unrounded);
        assert_eq!(
            output(0).size.height,
            px(root_height),
            "{display:?} fixed {fixed} fraction {fraction}"
        );
        assert_eq!(output(1).location.y, S::ZERO);
        assert_eq!(
            output(2).location.y,
            px(second_start),
            "{display:?} fixed {fixed} fraction {fraction}"
        );
        if !definite {
            for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
                let intrinsic = compute_layout(&tree, 0, request(operation)).unwrap();
                assert_eq!(
                    readback::single_box_output(&intrinsic, 0, Phase::Unrounded)
                        .size
                        .height,
                    px(root_height),
                    "{display:?} {operation:?} fixed {fixed} fraction {fraction}"
                );
            }
        }
    }
}

#[test]
fn grid_cyclic_gap_preserves_fixed_term() {
    assert_cyclic_gap_preserves_fixed_term::<f32>(Display::Grid);
    assert_cyclic_gap_preserves_fixed_term::<f64>(Display::Grid);
}

#[test]
fn grid_lanes_cyclic_gap_preserves_fixed_term() {
    assert_cyclic_gap_preserves_fixed_term::<f32>(Display::GridLanes);
    assert_cyclic_gap_preserves_fixed_term::<f64>(Display::GridLanes);
}

fn assert_contained_inline_gap_clamps_resolved_range<S: LayoutScalar>(display: Display) {
    let px = S::from_f64;
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Indefinite,
        SizingOperation::Normal,
    )));
    for (fixed, second_start) in [(-4.0, 11.0), (-8.0, 10.0)] {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
                    size_containment: SizeContainment::InlineAxis,
                    grid_template_columns: vec![TrackComponentOf::px(px(10.0)); 2],
                    grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                    gap: Size::new(
                        LengthOf::value(
                            LengthPercentageOf::from_coefficients(px(fixed), px(0.25)).unwrap(),
                        ),
                        LengthOf::ZERO,
                    ),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(px(10.0))),
                    grid_column: GridPlacement::try_line(2).unwrap(),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 0, request)
            .unwrap_or_else(|error| panic!("{display:?} fixed {fixed}: {error:?}"));
        let output = |node| readback::single_box_output(&batch, node, Phase::Unrounded);
        assert_eq!(output(0).size.width, px(20.0));
        assert_eq!(output(1).location.x, px(second_start));
    }
}

#[test]
fn grid_contained_inline_gap_clamps_resolved_range() {
    assert_contained_inline_gap_clamps_resolved_range::<f32>(Display::Grid);
    assert_contained_inline_gap_clamps_resolved_range::<f64>(Display::Grid);
}

#[test]
fn grid_lanes_contained_inline_gap_clamps_resolved_range() {
    assert_contained_inline_gap_clamps_resolved_range::<f32>(Display::GridLanes);
    assert_contained_inline_gap_clamps_resolved_range::<f64>(Display::GridLanes);
}

fn assert_nested_lanes_gap_preserves_fixed_term<S: LayoutScalar>() {
    let px = S::from_f64;
    let axes = Size::splat(AxisSizingRequestOf::new(
        AvailableSpaceOf::Indefinite,
        SizingOperation::MaxContent,
    ));
    let request = crate::LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            axes,
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    );
    for (fixed, fraction, second_width) in [
        (4.0, 0.0, 22.0),
        (0.0, 0.25, 20.0),
        (4.0, 0.25, 22.0),
        (-8.0, 0.25, 20.0),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 3])
            .children(1, [2])
            .children(2, [])
            .children(3, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::GridLanes,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
                    grid_template_columns: vec![TrackComponentOf::AUTO; 2],
                    grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                    gap: Size::splat(LengthOf::ZERO),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_column: GridPlacement::try_line_span(1, 2).unwrap(),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    grid_template_columns: subgrid_track_of(),
                    grid_template_rows: vec![TrackComponentOf::px(px(10.0))],
                    gap: Size::new(
                        LengthOf::value(
                            LengthPercentageOf::from_coefficients(px(fixed), px(fraction)).unwrap(),
                        ),
                        LengthOf::ZERO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    item_is_replaced: true,
                    justify_self: Some(AlignItems::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    grid_column: GridPlacement::try_line(2).unwrap(),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    justify_self: Some(AlignItems::Stretch),
                    ..NodeInputOf::default()
                },
            )
            .measure(2, Size::new(px(20.0), px(10.0)))
            .measure(3, Size::ZERO);
        let batch = compute_layout(&tree, 0, request).unwrap();
        let probe = readback::single_box_output(&batch, 3, Phase::Unrounded);
        assert_eq!(
            probe.size.width,
            px(second_width),
            "second track for fixed {fixed} fraction {fraction}"
        );
    }
}

#[test]
fn nested_lanes_gap_preserves_fixed_term() {
    assert_nested_lanes_gap_preserves_fixed_term::<f32>();
    assert_nested_lanes_gap_preserves_fixed_term::<f64>();
}
