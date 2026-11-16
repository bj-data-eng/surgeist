use super::fixtures::{
    SubgridChildParentContextInput, subgrid_axis_report, subgrid_track_of,
    with_projected_subgrid_child_input,
};
use super::*;
use crate::{MaxTrackSizingOf, MinTrackSizingOf, SizingCalculationOf};

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
                crate::LengthPercentageOf::from_coefficients(S::ZERO, scalar(largest)).unwrap(),
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
