//! Intrinsic track sizing of a Grid Lanes container whose auto-fit repetition
//! collapses empty tracks.

use super::fixtures::fri08_c03_auto_fit_batch;
use super::*;
use crate::test_support::readback::{self, Phase};

// Grid 3 section 3.3.1: tracks occupied by explicitly placed items are
// occupied, and with N the sum of the spans of the auto-placed items, the first
// N unoccupied tracks are occupied; the others are empty and, under auto-fit,
// collapse to a fixed 0px (Grid 2 section 7.2.3.2).
//
// Container 160px wide, `repeat(auto-fit, minmax(40px, auto))` gives four
// tracks. One item sits at line 1 and one auto-placed item spans two tracks with
// a 100px contribution: N is 2, so tracks 1 and 2 are occupied and track 3
// collapses.
//
// Grid 3 section 3.4: the auto-placed span-2 item is assumed at every start
// position, and Grid 2 section 12.5 sizes with each copy as a span-2 item.
// Start 0 covers the live tracks 0 and 1 and start 1 the live tracks 1 and 2;
// start 2 would cover the collapsed track 3, a fixed 0px, so it is no
// candidate (`lane_candidate_is_active`, reached through
// `active_candidate_starts`).
//
// The minimums are fixed 40px, so steps 3.1 to 3.3 raise no base sizes. Step 2
// sets the growth limit of track 0 from its explicit item (5px), raised to the
// base size 40; tracks 1 and 2 have no span-1 item and stay infinite. In step
// 3.5 the copy at start 0 needs 100 - (40 + 40) = 20 and track 0 is held at its
// finite limit, so track 1 takes all 20; the copy at start 1 shares its 20
// equally, 10 each to tracks 1 and 2. The planned increase of a track is the
// larger one (0, 20, 10), so the growth limits are 40, 60 and 50. Step 3.6
// changes nothing, since 40 + 60 and 60 + 50 already cover 100. Maximize tracks
// (section 12.6) raises the bases to the limits, 40, 60 and 50, and with the
// initial content distribution (stretch) section 12.8 divides the remaining
// 160 - 150 = 10 equally among the tracks with an auto maximum, the three live
// ones, 10/3 each: 43.33, 63.33 and 53.33.
//
// Item 2 is 0px high, so both starts of the auto-placed item have the same
// offset and the first, start 0, is chosen whatever the placement tolerance.
// The item is then 43.33 + 63.33 = 106.67 wide.
fn assert_auto_placed_span_ignores_collapsed_auto_fit_track<S: LayoutScalar>() {
    let px = S::from_f64;
    let container = Size::new(px(160.0), px(40.0));
    let group = vec![TrackComponentOf::from(TrackSizingOf::new(
        MinTrackSizingOf::Calculation(SizingCalculationOf::value(
            LengthPercentageOf::px(px(40.0)).unwrap(),
        )),
        MaxTrackSizingOf::Auto,
    ))];
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 3])
        .style(
            1,
            NodeInputOf {
                display: Display::GridLanes,
                size: container.map(PreferredSizeOf::px),
                grid_template_columns: vec![TrackComponentOf::Repeat(
                    TrackRepetitionOf::auto_fit_components(group).unwrap(),
                )],
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                grid_column: GridPlacement::try_line(1).unwrap(),
                size: Size::new(PreferredSizeOf::px(px(5.0)), PreferredSizeOf::px(px(0.0))),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                grid_column: GridPlacement::try_span(2).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .measure(3, Size::new(px(100.0), px(10.0)));
    let batch = fri08_c03_auto_fit_batch(&tree, container);
    let spanning = readback::single_box_output(&batch, 3, Phase::Unrounded);
    assert!(
        (spanning.size.width.to_f64() - (40.0 + 60.0 + 20.0 / 3.0)).abs() <= 1e-3,
        "{:?}",
        spanning.size
    );
    assert_eq!(spanning.location.x, S::ZERO);
}

#[test]
fn auto_placed_span_is_not_projected_over_a_collapsed_auto_fit_track() {
    assert_auto_placed_span_ignores_collapsed_auto_fit_track::<f32>();
    assert_auto_placed_span_ignores_collapsed_auto_fit_track::<f64>();
}
