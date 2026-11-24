use super::fixtures::{
    SubgridChildParentContextInput, subgrid_axis_report, subgrid_track_of,
    with_projected_subgrid_child_input,
};
use super::*;
use crate::{
    sizing::SizingCalculationOf,
    value::{MaxTrackSizingOf, MinTrackSizingOf},
};

// A parent's third column has a minimum function that overflows the scalar
// against the definite 100px basis. CSS Grid Level 2, section 9: a subgrid
// item inherits only the parent tracks its grid area spans. An item over the
// first column therefore inherits it, as the borrowed parent loan gives, and
// an item over all three columns reaches the third and is refused with the
// same error. Retaining the functions of every parent track instead refuses
// the first item too.
fn assert_share_reads_only_the_spanned_tracks<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let largest = if S::EPSILON.to_f64() > 1.0e-10 {
        f64::from(f32::MAX)
    } else {
        f64::MAX
    };
    let tracks = [
        TrackSizingOf::px(scalar(30.0)),
        TrackSizingOf::px(scalar(40.0)),
        TrackSizingOf::minmax(
            MinTrackSizingOf::Calculation(SizingCalculationOf::value(
                crate::value::LengthPercentageOf::from_coefficients(S::ZERO, scalar(largest))
                    .unwrap(),
            )),
            MaxTrackSizingOf::Auto,
        ),
    ];
    let parent_columns = [scalar(30.0), scalar(40.0), scalar(20.0)];
    let parent_rows = [scalar(50.0)];
    let parent_names_columns = named::NamedGridLines::new(GridAxisKind::Column, 3);
    let parent_names_rows = named::NamedGridLines::new(GridAxisKind::Row, 1);
    let parent_style = NodeInputOf::<S> {
        display: Display::Grid,
        ..NodeInputOf::default()
    };
    let child_style = NodeInputOf::<S> {
        display: Display::Grid,
        grid_template_columns: subgrid_track_of(),
        grid_template_rows: subgrid_track_of(),
        ..NodeInputOf::default()
    };
    let item = SubgridItemReport {
        node: 1u32,
        column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
        row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
    };
    let fixture = |column_end: usize| SubgridChildParentContextInput {
        item,
        child_style: &child_style,
        area: GridArea {
            row: 0,
            column: 0,
            row_end: 1,
            column_end,
            size: LogicalSizeOf::new(scalar(30.0), scalar(50.0)),
        },
        content_box_size: Size::new(scalar(30.0), scalar(50.0)),
        columns: &parent_columns,
        rows: &parent_rows,
        minimum_functions: GridMinimumParentFunctions {
            columns: Some(GridMinimumTrackFunctions::new(&tracks, Some(scalar(100.0)))),
            rows: None,
        },
        gap: LogicalSizeOf::new(S::ZERO, S::ZERO),
        parent_named_columns: &parent_names_columns,
        parent_named_rows: &parent_names_rows,
        parent_area_facts: None,
        margin: Edges::all(Some(S::ZERO)),
        border: Edges::ZERO,
        padding: Edges::ZERO,
    };

    for (column_end, refused) in [(1, false), (3, true)] {
        with_projected_subgrid_child_input(fixture(column_end), |input| {
            let borrowed = super::child::subgrid_child_parent_context(input);
            assert_eq!(borrowed.is_err(), refused, "columns 0..{column_end}");
            let share = SubgridParentShare::new(&input, None, None, None)
                .expect("the share reads no track function");
            let loan = SubgridItemLoan::new(share, &input);
            assert_eq!(
                super::child::subgrid_child_parent_context(loan.input()),
                borrowed,
                "columns 0..{column_end}"
            );
            // The contrast: the retention of every track refuses either item.
            assert!(OwnedSubgridContext::new(input).is_err());
        });
    }
}

#[test]
fn subgrid_share_reads_only_the_spanned_tracks_f32() {
    assert_share_reads_only_the_spanned_tracks::<f32>();
}

#[test]
fn subgrid_share_reads_only_the_spanned_tracks_f64() {
    assert_share_reads_only_the_spanned_tracks::<f64>();
}

// The parent context an intrinsic probe or contribution of a subgrid item owns
// reads the parent's minimum functions as the borrowed loan does: the item
// that spans only the first column inherits it, and the item that spans the
// third is refused with the same error, when its context is built. Making the
// request refuses the first item too if it retains every track's function.
fn assert_intrinsic_request_reads_only_the_spanned_tracks<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let largest = if S::EPSILON.to_f64() > 1.0e-10 {
        f64::from(f32::MAX)
    } else {
        f64::MAX
    };
    let tracks = [
        TrackSizingOf::px(scalar(30.0)),
        TrackSizingOf::px(scalar(40.0)),
        TrackSizingOf::minmax(
            MinTrackSizingOf::Calculation(SizingCalculationOf::value(
                crate::value::LengthPercentageOf::from_coefficients(S::ZERO, scalar(largest))
                    .unwrap(),
            )),
            MaxTrackSizingOf::Auto,
        ),
    ];
    let parent_columns = [scalar(30.0), scalar(40.0), scalar(20.0)];
    let parent_rows = [scalar(50.0)];
    let parent_names_columns = named::NamedGridLines::new(GridAxisKind::Column, 3);
    let parent_names_rows = named::NamedGridLines::new(GridAxisKind::Row, 1);
    let parent_style = NodeInputOf::<S> {
        display: Display::Grid,
        ..NodeInputOf::default()
    };
    let child_style = NodeInputOf::<S> {
        display: Display::Grid,
        grid_template_columns: subgrid_track_of(),
        grid_template_rows: subgrid_track_of(),
        ..NodeInputOf::default()
    };
    let item = SubgridItemReport {
        node: 1u32,
        column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
        row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
    };
    let fixture = |column_end: usize| SubgridChildParentContextInput {
        item,
        child_style: &child_style,
        area: GridArea {
            row: 0,
            column: 0,
            row_end: 1,
            column_end,
            size: LogicalSizeOf::new(scalar(30.0), scalar(50.0)),
        },
        content_box_size: Size::new(scalar(30.0), scalar(50.0)),
        columns: &parent_columns,
        rows: &parent_rows,
        minimum_functions: GridMinimumParentFunctions {
            columns: Some(GridMinimumTrackFunctions::new(&tracks, Some(scalar(100.0)))),
            rows: None,
        },
        gap: LogicalSizeOf::new(S::ZERO, S::ZERO),
        parent_named_columns: &parent_names_columns,
        parent_named_rows: &parent_names_rows,
        parent_area_facts: None,
        margin: Edges::all(Some(S::ZERO)),
        border: Edges::ZERO,
        padding: Edges::ZERO,
    };
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let constants = Constants {
        child_orthogonal_fallback: OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unknown),
        ),
        flow_axes: flow,
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
    let compute_input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(flow, ParentFormattingContext::Grid),
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );

    for (column_end, refused) in [(1, false), (3, true)] {
        for run in [ContextRun::Probe, ContextRun::Contribution] {
            with_projected_subgrid_child_input(fixture(column_end), |input| {
                let borrowed = super::child::subgrid_child_parent_context(
                    input.for_scrollbar_pass(&constants),
                );
                assert_eq!(borrowed.is_err(), refused, "columns 0..{column_end}");
                let request = GridContextRequest::intrinsic(
                    run,
                    1u32,
                    compute_input,
                    Size::NONE,
                    GridSizingStage::COMPLETE,
                    GridIntrinsicProbeKind::AllocatedPrefix,
                    input,
                    None,
                    None,
                    flow,
                );
                let built = request
                    .parent_context
                    .build::<core::convert::Infallible>(0u32, 1u32, &constants);
                assert_eq!(built.is_err(), refused, "columns 0..{column_end}");
                if let (Ok(built), Ok(borrowed)) = (built, borrowed) {
                    assert_eq!(built, borrowed, "columns 0..{column_end}");
                }
            });
        }
    }
}

#[test]
fn intrinsic_request_reads_only_the_spanned_tracks_f32() {
    assert_intrinsic_request_reads_only_the_spanned_tracks::<f32>();
}

#[test]
fn intrinsic_request_reads_only_the_spanned_tracks_f64() {
    assert_intrinsic_request_reads_only_the_spanned_tracks::<f64>();
}

// The parent facts of two shares are the same when the tracks, gap and names
// are the same and a used geometry that one lacks is the uniform geometry of
// its tracks, whichever share lacks it. A geometry with a gutter is not
// uniform.
fn assert_shares_compare_a_missing_geometry_as_uniform<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let parent_columns = [scalar(30.0), scalar(40.0)];
    let parent_rows = [scalar(50.0)];
    let parent_names_columns = named::NamedGridLines::new(GridAxisKind::Column, 2);
    let parent_names_rows = named::NamedGridLines::new(GridAxisKind::Row, 1);
    let parent_style = NodeInputOf::<S> {
        display: Display::Grid,
        ..NodeInputOf::default()
    };
    let child_style = NodeInputOf::<S> {
        display: Display::Grid,
        grid_template_columns: subgrid_track_of(),
        grid_template_rows: subgrid_track_of(),
        ..NodeInputOf::default()
    };
    let fixture = SubgridChildParentContextInput {
        item: SubgridItemReport {
            node: 1u32,
            column: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_style, &child_style, GridAxisKind::Row),
        },
        child_style: &child_style,
        area: GridArea {
            row: 0,
            column: 0,
            row_end: 1,
            column_end: 2,
            size: LogicalSizeOf::new(scalar(70.0), scalar(50.0)),
        },
        content_box_size: Size::new(scalar(70.0), scalar(50.0)),
        columns: &parent_columns,
        rows: &parent_rows,
        minimum_functions: GridMinimumParentFunctions {
            columns: None,
            rows: None,
        },
        gap: LogicalSizeOf::new(S::ZERO, S::ZERO),
        parent_named_columns: &parent_names_columns,
        parent_named_rows: &parent_names_rows,
        parent_area_facts: None,
        margin: Edges::all(Some(S::ZERO)),
        border: Edges::ZERO,
        padding: Edges::ZERO,
    };
    with_projected_subgrid_child_input(fixture, |input| {
        let uniform_columns =
            UsedGridAxisGeometryOf::new(parent_columns.to_vec(), Vec::new(), S::ZERO);
        let uniform_rows = UsedGridAxisGeometryOf::new(parent_rows.to_vec(), Vec::new(), S::ZERO);
        let gutter_columns =
            UsedGridAxisGeometryOf::new(parent_columns.to_vec(), vec![true], S::ZERO);
        let share = |columns: Option<&UsedGridAxisGeometryOf<S>>,
                     rows: Option<&UsedGridAxisGeometryOf<S>>| {
            SubgridParentShare::new(&input, columns, rows, None).expect("the share copies")
        };
        let lacking = share(None, None);
        let uniform = share(Some(&uniform_columns), Some(&uniform_rows));
        let gutter = share(Some(&gutter_columns), Some(&uniform_rows));
        assert!(lacking.same_parent_facts(&uniform));
        assert!(uniform.same_parent_facts(&lacking));
        assert!(!lacking.same_parent_facts(&gutter));
        assert!(!gutter.same_parent_facts(&lacking));
    });
}

#[test]
fn subgrid_shares_compare_a_missing_geometry_as_uniform_f32() {
    assert_shares_compare_a_missing_geometry_as_uniform::<f32>();
}

#[test]
fn subgrid_shares_compare_a_missing_geometry_as_uniform_f64() {
    assert_shares_compare_a_missing_geometry_as_uniform::<f64>();
}
