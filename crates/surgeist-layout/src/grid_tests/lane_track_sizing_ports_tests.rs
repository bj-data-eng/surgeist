//! Intrinsic track sizing behaviours of the standalone lane chain, restated
//! through `compute_layout` so that they hold for the driver.
//!
//! Every expectation is the value the standalone test it replaces asserted,
//! and each one is also what Grid 2 section 12.5 gives for the same tracks and
//! items (derivations in the comments), so the expectations do not depend on
//! either implementation.

use super::fixtures::{fri04_c03_grid_track_nested, invalid_numeric_lp, lp};
use super::*;
use crate::test_support::lane_track_sizes::{
    ContainerWidth, TrackItem, column_track_sizes, try_column_track_sizes,
};

fn lanes_sizes<S: LayoutScalar>(
    tracks: Vec<TrackSizingOf<S>>,
    gap: f64,
    items: &[TrackItem],
) -> Vec<S> {
    column_track_sizes(
        Display::GridLanes,
        ContainerWidth::Definite(0.0),
        tracks,
        gap,
        items,
    )
}

fn assert_close<S: LayoutScalar>(actual: &[S], expected: &[f64]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{actual:?} against {expected:?}"
    );
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual.to_f64() - expected).abs() <= 1e-4,
            "{actual:?} against {expected:?}"
        );
    }
}

fn flex<S: LayoutScalar>(factor: f64) -> TrackSizingOf<S> {
    TrackSizingOf::flex(TrackFlexFactorOf::try_new(S::from_f64(factor)).expect("valid flex factor"))
}

// Two auto-placed span-2 items over three auto tracks with a 10px gap. Each is
// projected to starts 0 and 1 (Grid 3 section 3.4); step 3.1 of Grid 2 section
// 12.5 gives each start's tracks (30 - 10) / 2 = 10 at most, so all three
// tracks are 10.
fn assert_equivalent_auto_span_items_size_every_track<S: LayoutScalar>() {
    let sizes = lanes_sizes::<S>(
        vec![TrackSizingOf::AUTO; 3],
        10.0,
        &[
            TrackItem::auto(2, 20.0, 50.0).with_automatic_minimum(),
            TrackItem::auto(2, 30.0, 60.0).with_automatic_minimum(),
        ],
    );
    assert_close(&sizes, &[10.0, 10.0, 10.0]);
}

#[test]
fn equivalent_auto_span_items_size_every_track_through_compute_layout() {
    assert_equivalent_auto_span_items_size_every_track::<f32>();
    assert_equivalent_auto_span_items_size_every_track::<f64>();
}

// Grid 3 section 3.4: an auto-placed item contributes to every track it could
// be placed in. A span-1 item with a 10px minimum sizes all four tracks to 10;
// a span-2 item with a 30px minimum is assumed at every start, so each track
// receives 15 from the start that covers it.
#[test]
fn auto_placed_items_contribute_at_every_possible_start() {
    let tracks = vec![TrackSizingOf::<f32>::AUTO; 4];
    let span_one = lanes_sizes(
        tracks.clone(),
        0.0,
        &[TrackItem::auto(1, 10.0, 20.0).with_automatic_minimum()],
    );
    assert_close(&span_one, &[10.0; 4]);
    let span_two = lanes_sizes(
        tracks,
        0.0,
        &[TrackItem::auto(2, 30.0, 40.0).with_automatic_minimum()],
    );
    assert_close(&span_two, &[15.0; 4]);
}

// A definite item over lines 2 to 4 of four auto tracks is sized exactly there
// (Grid 3 section 3.4, first bullet): tracks 1 and 2 (zero-based) share its
// 20px minimum equally and no other track grows.
#[test]
fn definite_spans_contribute_only_to_their_tracks() {
    let sizes = lanes_sizes::<f32>(
        vec![TrackSizingOf::AUTO; 4],
        0.0,
        &[TrackItem::definite(1, 2, 20.0, 30.0).with_automatic_minimum()],
    );
    assert_close(&sizes, &[0.0, 10.0, 10.0, 0.0]);
}

// One span-3 item with a 100px minimum over auto, a 20px fixed track and auto,
// gap 10: step 3.1 distributes 100 - (0 + 20 + 0 + 2 * 10) = 60 equally to the
// two intrinsic-minimum tracks.
fn assert_fixed_track_between_auto_tracks_with_gap<S: LayoutScalar>() {
    let sizes = lanes_sizes::<S>(
        vec![
            TrackSizingOf::AUTO,
            TrackSizingOf::px(S::from_f64(20.0)),
            TrackSizingOf::AUTO,
        ],
        10.0,
        &[TrackItem::auto(3, 120.0, 160.0).with_min_size(100.0)],
    );
    assert_close(&sizes, &[30.0, 20.0, 30.0]);
}

#[test]
fn fixed_track_between_auto_tracks_shares_the_remaining_minimum() {
    assert_fixed_track_between_auto_tracks_with_gap::<f32>();
    assert_fixed_track_between_auto_tracks_with_gap::<f64>();
}

// Grid 2 section 12.5 step 2: a min-content minimum takes the largest min-content
// contribution of its span-1 items, 100, whatever the item's specified minimum
// size (12) and max-content contribution (120).
#[test]
fn min_content_minimum_takes_the_min_content_contribution() {
    let track = TrackSizing::new(MinTrackSizing::MIN_CONTENT, MaxTrackSizing::MAX_CONTENT);
    let sizes = lanes_sizes(
        vec![track],
        10.0,
        &[TrackItem::auto(1, 100.0, 120.0).with_min_size(12.0)],
    );
    assert_close(&sizes, &[100.0]);
}

// A nested calculation clamp(20, max(40, min(35, 45)), 60) is 40.
#[test]
fn nested_calculation_track_base_resolves_in_a_lanes_container() {
    let track = TrackSizing::new(
        MinTrackSizing::Calculation(fri04_c03_grid_track_nested(20.0, 40.0, 60.0)),
        MaxTrackSizing::Auto,
    );
    let sizes = lanes_sizes(vec![track], 0.0, &[]);
    assert_close(&sizes, &[40.0]);
}

fn assert_invalid_track_numeric<S: LayoutScalar>(
    calculation: SizingCalculationOf<S>,
    width: f64,
    expected: f64,
) {
    let error = try_column_track_sizes::<S>(
        Display::GridLanes,
        ContainerWidth::Definite(width),
        vec![TrackSizingOf::new(
            MinTrackSizingOf::Calculation(calculation),
            MaxTrackSizingOf::Auto,
        )],
        0.0,
        &[],
    )
    .expect_err("an affine track size that overflows is invalid input");
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
                if value.to_f64() == expected
        ),
        "{error:?}"
    );
}

#[test]
fn overflowing_affine_track_sizes_are_invalid_input_in_both_directions() {
    assert_invalid_track_numeric::<f32>(
        SizingCalculation::value(invalid_numeric_lp()),
        2.0,
        f64::INFINITY,
    );
    assert_invalid_track_numeric::<f32>(
        SizingCalculation::value(lp(-f32::MAX, -1.0)),
        f64::from(f32::MAX),
        f64::NEG_INFINITY,
    );
    assert_invalid_track_numeric::<f64>(
        SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(f64::MAX, 1.0).expect("finite coefficients"),
        ),
        f64::MAX,
        f64::INFINITY,
    );
    assert_invalid_track_numeric::<f64>(
        SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(-f64::MAX, -1.0).expect("finite coefficients"),
        ),
        f64::MAX,
        f64::NEG_INFINITY,
    );
}

// An f64 container: one auto track, a definite span-1 item with a 9.5
// automatic minimum and a 1.25 gap.
#[test]
fn f64_lanes_container_sizes_an_auto_track_from_its_item() {
    let sizes = lanes_sizes::<f64>(
        vec![TrackSizingOf::AUTO],
        1.25,
        &[TrackItem::definite(0, 1, 9.5, 14.25).with_automatic_minimum()],
    );
    assert_close(&sizes, &[9.5]);
}

// Grid 2 section 12.5.1 step 2.2 distributes the space equally, not to equal
// totals: bases 1.25 and 3.75 (span-1 items) and a span-2 minimum of 12 leave
// 12 - 5 = 7 to share, 3.5 each, so the tracks are 4.75 and 7.25.
// (The removed distribute_intrinsic_span test asserted [6, 6] by leveling.)
#[test]
fn spanning_item_adds_its_extra_space_in_equal_increments() {
    let sizes = lanes_sizes::<f64>(
        vec![TrackSizingOf::AUTO; 2],
        0.0,
        &[
            TrackItem::definite(0, 1, 1.25, 1.25).with_automatic_minimum(),
            TrackItem::definite(1, 1, 3.75, 3.75).with_automatic_minimum(),
            TrackItem::definite(0, 2, 12.0, 12.0).with_automatic_minimum(),
        ],
    );
    assert_close(&sizes, &[4.75, 7.25]);
}

// Flexible tracks under a max-content container. Grid 2 section 12.7 (flex
// fraction from the spanning item's max-content contribution).
fn grid_flex_sizes<S: LayoutScalar>(
    display: Display,
    tracks: Vec<TrackSizingOf<S>>,
    items: &[TrackItem],
) -> Vec<S> {
    column_track_sizes(display, ContainerWidth::MaxContent, tracks, 0.0, items)
}

#[test]
fn span_contribution_over_flexible_tracks_is_shared_by_flex_factor() {
    for display in [Display::Grid, Display::GridLanes] {
        let sizes = grid_flex_sizes::<f32>(
            display,
            vec![flex(1.0), flex(2.0)],
            &[TrackItem::definite(0, 2, 60.0, 60.0)],
        );
        assert_close(&sizes, &[20.0, 40.0]);
    }
}

#[test]
fn span_contribution_over_flexible_tracks_is_net_of_non_flexible_base_sizes() {
    for display in [Display::Grid, Display::GridLanes] {
        let sizes = grid_flex_sizes::<f32>(
            display,
            vec![TrackSizingOf::MIN_CONTENT, flex(1.0)],
            &[
                TrackItem::definite(0, 1, 10.0, 10.0),
                TrackItem::definite(0, 2, 40.0, 40.0),
            ],
        );
        assert_close(&sizes, &[10.0, 30.0]);
    }
}

#[test]
fn span_contribution_over_sub_one_flex_factors_after_a_fixed_track_keeps_their_ratio() {
    for display in [Display::Grid, Display::GridLanes] {
        let sizes = grid_flex_sizes::<f32>(
            display,
            vec![TrackSizingOf::px(9.0), flex(0.5), flex(0.5)],
            &[TrackItem::definite(0, 3, 18.0, 18.0)],
        );
        assert_close(&sizes, &[9.0, 4.5, 4.5]);
    }
}

#[test]
fn max_content_span_prefers_tracks_with_a_max_content_minimum() {
    let tracks = vec![
        TrackSizingOf::<f32>::MIN_CONTENT,
        TrackSizingOf::MAX_CONTENT,
        TrackSizingOf::px(10.0),
    ];
    let items = [
        TrackItem::definite(0, 1, 80.0, 80.0),
        TrackItem::definite(1, 1, 80.0, 80.0),
        TrackItem::definite(0, 3, 0.0, 320.0),
    ];
    for display in [Display::Grid, Display::GridLanes] {
        let sizes = column_track_sizes(
            display,
            ContainerWidth::Definite(0.0),
            tracks.clone(),
            0.0,
            &items,
        );
        assert_close(&sizes, &[80.0, 230.0, 10.0]);
    }
}

#[test]
fn max_content_span_prefers_a_max_content_maximum_over_a_min_content_maximum() {
    let tracks = vec![
        TrackSizingOf::<f32>::MAX_CONTENT,
        TrackSizing::minmax(MinTrackSizing::MAX_CONTENT, MaxTrackSizing::MIN_CONTENT),
    ];
    let items = [
        TrackItem::definite(0, 1, 40.0, 40.0),
        TrackItem::definite(1, 1, 20.0, 20.0),
        TrackItem::definite(0, 2, 0.0, 80.0),
    ];
    for display in [Display::Grid, Display::GridLanes] {
        let sizes = column_track_sizes(
            display,
            ContainerWidth::Definite(0.0),
            tracks.clone(),
            0.0,
            &items,
        );
        assert_close(&sizes, &[60.0, 20.0]);
    }
}
