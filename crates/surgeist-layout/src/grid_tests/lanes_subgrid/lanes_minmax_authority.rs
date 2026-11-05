// Private child of grid_tests/lanes_subgrid_tests.rs; no final stacking query.
// Native measurement varies by actual min/max query; overflow is independent.
use super::*;
use crate::engine::contracts::Compute;
use crate::test_support::layout_tree::OracleMeasurementOf;

#[derive(Clone, Copy, Debug)]
enum Demand {
    MinContent,
    MaxContent,
}
impl Demand {
    fn track<S: LayoutScalar>(self) -> TrackSizingOf<S> {
        match self {
            Self::MinContent => TrackSizingOf::MIN_CONTENT,
            Self::MaxContent => TrackSizingOf::MAX_CONTENT,
        }
    }
    fn expected(self) -> f64 {
        match self {
            Self::MinContent => 20.0,
            Self::MaxContent => 30.0,
        }
    }
}

// Independent physical table for the two authored modes. Direction changes
// progression, not the physical dimension of a min/max size or availability.
fn physical_pair<T>(mode: WritingMode, axis: GridAxisKind, selected: T, other: T) -> Size<T> {
    match (mode, axis) {
        (WritingMode::HorizontalTb, GridAxisKind::Column)
        | (WritingMode::VerticalRl, GridAxisKind::Row) => Size::new(selected, other),
        (WritingMode::HorizontalTb, GridAxisKind::Row)
        | (WritingMode::VerticalRl, GridAxisKind::Column) => Size::new(other, selected),
        _ => unreachable!("the fixture authors only horizontal-tb and vertical-rl"),
    }
}
fn response<S: LayoutScalar>(
    mode: WritingMode,
    axis: GridAxisKind,
    border: f64,
    overflow: f64,
) -> ComputeOutputOf<S> {
    ComputeOutputOf::from_sizes(
        physical_pair(mode, axis, S::from_f64(border), S::from_f64(10.0)),
        physical_pair(mode, axis, S::from_f64(overflow), S::from_f64(10.0)),
    )
}

fn differential<S: LayoutScalar>(axis: GridAxisKind, overflow: Overflow, demand: Demand) {
    for mode in [WritingMode::HorizontalTb, WritingMode::VerticalRl] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let min_available = physical_pair(
                mode,
                axis,
                AxisSizingRequestOf::<S>::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MinContent,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            );
            let max_available = Size::splat(AxisSizingRequestOf::<S>::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            ));
            let minimum_known = physical_pair(mode, axis, Some(S::ZERO), None);
            let minimum_response = response::<S>(mode, axis, 0.0, 80.0);
            let min_response = response::<S>(mode, axis, 20.0, 80.0);
            let max_response = response::<S>(mode, axis, 30.0, 90.0);
            assert_ne!(min_response.size, max_response.size);
            assert_ne!(min_response.size, min_response.content_size);
            assert_ne!(max_response.size, max_response.content_size);
            assert_eq!(min_response.baselines(), BaselinesOf::NONE);
            assert_eq!(max_response.baselines(), BaselinesOf::NONE);
            assert_eq!(min_response.named_baselines, NamedBaselinesOf::unknown());
            assert_eq!(max_response.named_baselines, NamedBaselinesOf::unknown());

            let track = demand.track::<S>();
            let (columns, rows, auto_flow) = match axis {
                GridAxisKind::Column => (
                    vec![TrackComponentOf::Track(track.clone())],
                    vec![TrackComponentOf::px(S::from_f64(10.0))],
                    GridAutoFlow::Row,
                ),
                GridAxisKind::Row => (
                    vec![TrackComponentOf::px(S::from_f64(10.0))],
                    vec![TrackComponentOf::Track(track.clone())],
                    GridAutoFlow::Column,
                ),
            };
            let line_one = GridPlacement::try_line(1).expect("one is a valid authored line");
            let mut tree = OracleTreeOf::<S>::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::GridLanes,
                        writing_mode: mode,
                        direction,
                        grid_auto_flow: auto_flow,
                        grid_template_columns: columns,
                        grid_template_rows: rows,
                        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                        align_items: Some(AlignItems::Start),
                        justify_items: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: mode,
                        direction,
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                        grid_column: line_one,
                        grid_row: line_one,
                        overflow: computed_overflow(overflow, overflow),
                        ..NodeInputOf::default()
                    },
                )
                // Each real role has an exact query; no wildcard shadows a response.
                .measure_when(
                    1,
                    OracleMeasurementOf::new(minimum_response)
                        .run_mode(RunMode::ComputeSize)
                        .known(minimum_known)
                        .parent(Size::NONE)
                        .operations((min_available).map(|axis| Some(axis.operation())))
                        .spaces((min_available).map(|axis| {
                            (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                        })),
                )
                .measure_when(
                    1,
                    OracleMeasurementOf::new(min_response)
                        .run_mode(RunMode::ComputeSize)
                        .known(Size::NONE)
                        .parent(Size::NONE)
                        .operations((min_available).map(|axis| Some(axis.operation())))
                        .spaces((min_available).map(|axis| {
                            (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                        })),
                )
                .measure_when(
                    1,
                    OracleMeasurementOf::new(max_response)
                        .run_mode(RunMode::ComputeSize)
                        .known(Size::NONE)
                        .parent(Size::NONE)
                        .operations((max_available).map(|axis| Some(axis.operation())))
                        .spaces((max_available).map(|axis| {
                            (axis.operation() == SizingOperation::Normal).then_some(axis.space())
                        })),
                );
            assert!(tree.node_input(1).size.width.is_auto());
            assert!(tree.node_input(1).size.height.is_auto());

            // Same real owner assembly as the maintained order-modified lanes
            // fixture. One child, one actual declared span, no baseline demand.
            let sizes = {
                let mut compute = tree.borrow_compute();
                let child_inputs = vec![grid_child_input!(compute.node_input(1))];
                let placements = GridPlacementContext::new_with_child_inputs(
                    vec![1],
                    vec![ResolvedGridItemPlacement {
                        column: line_one,
                        row: line_one,
                        absolute_column: line_one,
                        absolute_row: line_one,
                        in_flow: true,
                    }],
                    child_inputs,
                )
                .with_order_modified_indexes(vec![SourceIndex::new(0)]);
                let constants = Constants {
                    child_orthogonal_fallback: OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                    ),
                    flow_axes: FlowAxes::new(mode, direction),
                    explicit_definite_content_size: Size::NONE,
                    node_outer_size: Size::NONE,
                    node_inner_size: Size::NONE,
                    node_min_size: Size::NONE,
                    fallback_minimum_outer: Size::splat(FixedFallbackSource::Unqualified),
                    fallback_maximum_outer: Size::splat(FixedFallbackSource::Unqualified),
                    node_max_size: Size::NONE,
                    available_inner_size: Size::NONE,
                    content_box_inset: Edges::ZERO,
                    scrollbar_gutter: Edges::ZERO,
                    padding: Edges::ZERO,
                    border: Edges::ZERO,
                };
                let container_input = compute.node_input(0);
                let container_style = grid_container_projection!(container_input);
                let subgrid_report = collect_subgrid_report(&container_style, &placements);
                let column_tracks = expand_track_components(
                    &container_input.grid_template_columns,
                    None,
                    S::ZERO,
                    None,
                )
                .unwrap();
                let row_tracks = expand_track_components(
                    &container_input.grid_template_rows,
                    None,
                    S::ZERO,
                    None,
                )
                .unwrap();
                let tracks = match axis {
                    GridAxisKind::Column => column_tracks.as_slice(),
                    GridAxisKind::Row => row_tracks.as_slice(),
                };
                let lines = GridLines {
                    column_explicit_start: 0,
                    column_explicit_count: 1,
                    row_explicit_start: 0,
                    row_explicit_count: 1,
                };
                let named_columns =
                    named::NamedGridLines::new(GridAxisKind::Column, column_tracks.len());
                let named_rows = named::NamedGridLines::new(GridAxisKind::Row, row_tracks.len());
                let grid = IntrinsicGrid {
                    style: &container_style,
                    constants: &constants,
                    sizing_flow_axes: constants.flow_axes,
                    track_basis: constants.flow_axes.logical_size(constants.node_inner_size),
                    minimum_functions: GridMinimumParentFunctions {
                        columns: Some(GridMinimumTrackFunctions::new(&column_tracks, None)),
                        rows: Some(GridMinimumTrackFunctions::new(&row_tracks, None)),
                    },
                    column_tracks: &column_tracks,
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
                let rows = lane_intrinsic_contribution_rows(
                    &mut compute,
                    0,
                    LaneIntrinsicTrackSizeInput {
                        grid,
                        frame: intrinsic_stage::IntrinsicStageFrame {
                            axis,
                            stage: GridSizingStage {
                                first_axis: axis,
                                through: GridSizingPhase::InitialFirst,
                            },
                            columns: None,
                            rows: None,
                            observed_axis: None,
                        },
                        container_style: &container_style,
                        constants: &constants,
                        axis,
                        tracks,
                        gap: S::ZERO,
                        gutters: None,
                        lines,
                        placements: &placements,
                        subgrid_report: &subgrid_report,
                    },
                )
                .expect("actual native intrinsic query succeeds")
                .expect("one real child occupies its declared one-track span");
                resolve_grid_contribution_rows(
                    SpanningTrackContext {
                        tracks,
                        basis: None,
                        available: GridTrackSizingConstraint {
                            operation: SizingOperation::MinContent,
                            available_extent: None,
                        },
                        gap: S::ZERO,
                        gutters: None,
                        base_floors: &[],
                    },
                    &rows,
                )
                .expect("actual native rows resolve")
                .base_sizes
            };

            let requests = tree.inputs(1);
            assert!(
                requests
                    .iter()
                    .any(|input| input.sizing_request().axes() == min_available
                        && input.known() == Size::NONE),
                "{axis:?} {overflow:?} {demand:?} {mode:?} {direction:?}: MIN_CONTENT query must reach the provider"
            );
            assert!(
                requests
                    .iter()
                    .any(|input| input.sizing_request().axes() == max_available
                        && input.known() == Size::NONE),
                "{axis:?} {overflow:?} {demand:?} {mode:?} {direction:?}: MAX_CONTENT query must reach the provider"
            );
            assert!(
                requests
                    .iter()
                    .any(|input| input.sizing_request().axes() == min_available
                        && input.known() == minimum_known),
                "the real Minimum query retains its resolved zero border"
            );
            for input in requests {
                assert_eq!(input.run_mode(), RunMode::ComputeSize);
                assert_eq!(input.sizing_mode(), SizingMode::InherentSize);
                assert_eq!(input.requested_axis(), RequestedAxis::Both);
                assert!(input.known() == Size::NONE || input.known() == minimum_known);
                if input.known() == minimum_known {
                    assert_eq!(input.sizing_request().axes(), min_available);
                }
                assert_eq!(input.parent(), Size::NONE);
                assert!(
                    input.sizing_request().axes() == min_available
                        || input.sizing_request().axes() == max_available
                );
            }
            // A sole min-content track uses queried border20. A sole
            // max-content track uses queried border30. Independent visible
            // overflow80/90 is not contribution authority; Clip agrees.
            assert_eq!(
                sizes,
                vec![S::from_f64(demand.expected())],
                "{axis:?} {overflow:?} {demand:?} {mode:?} {direction:?}: use each query's actual returned border, not overflow"
            );
        }
    }
}

macro_rules! lane_case {
    ($name:ident, $scalar:ty, $axis:expr, $overflow:expr, $demand:expr) => {
        #[test]
        fn $name() {
            differential::<$scalar>($axis, $overflow, $demand);
        }
    };
}
lane_case!(
    min_border_visible_column_f32,
    f32,
    GridAxisKind::Column,
    Overflow::Visible,
    Demand::MinContent
);
lane_case!(
    max_border_visible_column_f32,
    f32,
    GridAxisKind::Column,
    Overflow::Visible,
    Demand::MaxContent
);
lane_case!(
    min_border_clip_column_f32,
    f32,
    GridAxisKind::Column,
    Overflow::Clip,
    Demand::MinContent
);
lane_case!(
    max_border_clip_column_f32,
    f32,
    GridAxisKind::Column,
    Overflow::Clip,
    Demand::MaxContent
);
lane_case!(
    min_border_visible_row_f32,
    f32,
    GridAxisKind::Row,
    Overflow::Visible,
    Demand::MinContent
);
lane_case!(
    max_border_visible_row_f32,
    f32,
    GridAxisKind::Row,
    Overflow::Visible,
    Demand::MaxContent
);
lane_case!(
    min_border_clip_row_f32,
    f32,
    GridAxisKind::Row,
    Overflow::Clip,
    Demand::MinContent
);
lane_case!(
    max_border_clip_row_f32,
    f32,
    GridAxisKind::Row,
    Overflow::Clip,
    Demand::MaxContent
);
lane_case!(
    min_border_visible_column_f64,
    f64,
    GridAxisKind::Column,
    Overflow::Visible,
    Demand::MinContent
);
lane_case!(
    max_border_visible_column_f64,
    f64,
    GridAxisKind::Column,
    Overflow::Visible,
    Demand::MaxContent
);
lane_case!(
    min_border_clip_column_f64,
    f64,
    GridAxisKind::Column,
    Overflow::Clip,
    Demand::MinContent
);
lane_case!(
    max_border_clip_column_f64,
    f64,
    GridAxisKind::Column,
    Overflow::Clip,
    Demand::MaxContent
);
lane_case!(
    min_border_visible_row_f64,
    f64,
    GridAxisKind::Row,
    Overflow::Visible,
    Demand::MinContent
);
lane_case!(
    max_border_visible_row_f64,
    f64,
    GridAxisKind::Row,
    Overflow::Visible,
    Demand::MaxContent
);
lane_case!(
    min_border_clip_row_f64,
    f64,
    GridAxisKind::Row,
    Overflow::Clip,
    Demand::MinContent
);
lane_case!(
    max_border_clip_row_f64,
    f64,
    GridAxisKind::Row,
    Overflow::Clip,
    Demand::MaxContent
);
