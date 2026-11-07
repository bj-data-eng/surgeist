//! Equal flex shares must fit the definite budget in canonical track order.
use super::*;
use crate::test_support::readback::{self, Phase};

fn assert_native_equal_flex_budget<S: LayoutScalar>(
    count: usize,
    share: S,
    feasible_sum: S,
    published_edges: &[S],
) {
    let zero = S::ZERO;
    let one = S::from_f64(1.0);
    assert_eq!(published_edges.len(), count + 1);
    let last_node = u32::try_from(count).expect("bounded native fixture");
    let track = TrackComponentOf::minmax(
        MinTrackSizingOf::px(zero),
        MaxTrackSizingOf::flex(TrackFlexFactorOf::try_new(one).unwrap()),
    );
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, 1..=last_node)
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(PreferredSizeOf::px(one), PreferredSizeOf::px(one)),
                grid_template_columns: vec![track.clone(); count],
                grid_template_rows: vec![TrackComponentOf::px(one)],
                gap: Size::splat(LengthOf::px(zero)),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
    for (index, node) in (1..=last_node).enumerate() {
        let start = isize::try_from(index + 1).unwrap();
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                grid_column: GridPlacement::try_lines(start, start + 1).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                justify_self: Some(AlignItems::Stretch),
                align_self: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        );
    }
    assert_eq!(
        Traverse::children(&tree, 0).collect::<Vec<_>>(),
        (1..=last_node).collect::<Vec<_>>()
    );
    let root_input = tree.node_input(0);
    assert_eq!(
        root_input.size,
        Size::new(PreferredSizeOf::px(one), PreferredSizeOf::px(one))
    );
    assert_eq!(root_input.grid_template_columns, vec![track; count]);
    assert_eq!(
        root_input.grid_template_rows,
        vec![TrackComponentOf::px(one)]
    );
    assert_eq!(root_input.gap, Size::splat(LengthOf::px(zero)));
    for node in 0..=last_node {
        let input = tree.node_input(node);
        assert!(
            !tree.has_leaf_measurement(node),
            "native layout has no supplied measurement"
        );
        assert_eq!(input.margin, Edges::all(LengthAutoOf::px(zero)));
        assert_eq!(input.border, Edges::all(LengthOf::px(zero)));
        assert_eq!(input.padding, Edges::all(LengthOf::px(zero)));
        if node != 0 {
            assert_eq!(input.display, Display::Flex);
            assert_eq!(
                input.size,
                Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
            );
            assert_eq!(Traverse::child_count(&tree, node), 0);
        }
    }
    // Empty native children have no text, font, measurement or descendant
    // demand. Explicit zero track minima and zero gutters leave all1 for fr.
    let batch = compute_layout(&tree, 0, {
        let axes = Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(one).unwrap()),
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
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("the finite native grid completes");
    let root = readback::single_box_output(&batch, 0, Phase::Unrounded);
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(root.size, Size::splat(one));
    let mut cursor = zero;
    let mut actual_sum = zero;
    for (index, node) in (1..=last_node).enumerate() {
        let actual = readback::single_box_output(&batch, node, Phase::Unrounded);
        let right = cursor + share;
        assert_eq!(actual.source_index, SourceIndex::new(index));
        assert_eq!(
            (actual.margin, actual.border, actual.padding),
            (Edges::ZERO, Edges::ZERO, Edges::ZERO)
        );
        assert_eq!(
            actual.size,
            Size::new(share, one),
            "node{node} receives the greatest feasible represented equal share"
        );
        assert_eq!(
            actual.location,
            Point::new(cursor, zero),
            "node{node} follows canonical track order"
        );
        assert_eq!(actual.location.x + actual.size.width, right);
        actual_sum = actual_sum + actual.size.width;
        assert!(
            actual_sum <= one,
            "the actual left-fold allocation never exceeds the budget"
        );
        assert!(
            right <= one,
            "the actual ordered endpoint remains in the budget"
        );

        // Declared edges follow the existing scalar publication policy:
        // represented addition of0.5, then floor. Its near-half tie is
        // qualified independently below; the unrounded budget remains exact.
        let final_box = readback::single_box_output(&batch, node, Phase::Final);
        assert_eq!(final_box.source_index, SourceIndex::new(index));
        assert_eq!(final_box.location, Point::new(published_edges[index], zero));
        assert_eq!(
            final_box.size,
            Size::new(published_edges[index + 1] - published_edges[index], one)
        );
        cursor = right;
    }
    assert_eq!(actual_sum, feasible_sum);
    assert_eq!(cursor, feasible_sum);
    assert!(cursor <= root.size.width);
    assert_eq!(
        readback::single_box_output(&batch, 0, Phase::Final).size,
        Size::splat(one)
    );
}

#[test]
fn native_ten_equal_flex_tracks_conserve_the_f32_budget_before_rounding() {
    let feasible = f32::from_bits(0x3dcc_cccc);
    let next = f32::from_bits(0x3dcc_cccd);
    assert_eq!(next.to_bits(), feasible.to_bits() + 1);
    assert_eq!(next, 1.0_f32 / 10.0);
    let feasible_sum = (0..10).fold(0.0_f32, |sum, _| sum + feasible);
    let next_sum = (0..10).fold(0.0_f32, |sum, _| sum + next);
    assert_eq!(feasible_sum, 1.0);
    assert_eq!(next_sum.to_bits(), 0x3f80_0001);
    assert!(next_sum > 1.0);
    // Adjacent positive IEEE shares are ordered. The next share is the nominal
    // quotient and already overspends, so feasible is greatest at/below nominal.
    let fifth = (0..5).fold(0.0_f32, |sum, _| sum + feasible);
    assert_eq!(fifth.to_bits(), 0x3eff_ffff);
    assert!(fifth < 0.5);
    let previous_one = f32::from_bits(0x3f7f_ffff);
    assert_eq!(
        f64::from(fifth) + 0.5,
        (f64::from(previous_one) + 1.0) / 2.0,
        "the exact addition is halfway between adjacent represented values below/at1",
    );
    assert_eq!((fifth + 0.5_f32).to_bits(), 0x3f80_0000);
    assert_eq!(fifth.round(), 0.0);
    // The native positive-coordinate policy first adds0.5 in f32. At this
    // tie the addition rounds to even1, so the fifth published edge is1.
    assert_native_equal_flex_budget::<f32>(
        10,
        feasible,
        feasible_sum,
        &[0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    );
}

#[test]
fn native_nine_equal_flex_tracks_conserve_the_f64_budget_before_rounding() {
    let feasible = f64::from_bits(0x3fbc_71c7_1c71_c71b);
    let next = f64::from_bits(0x3fbc_71c7_1c71_c71c);
    assert_eq!(next.to_bits(), feasible.to_bits() + 1);
    assert_eq!(next, 1.0_f64 / 9.0);
    let feasible_sum = (0..9).fold(0.0_f64, |sum, _| sum + feasible);
    let next_sum = (0..9).fold(0.0_f64, |sum, _| sum + next);
    assert_eq!(feasible_sum.to_bits(), 0x3fef_ffff_ffff_fffd);
    assert!(feasible_sum <= 1.0);
    assert_eq!(next_sum.to_bits(), 0x3ff0_0000_0000_0001);
    assert!(next_sum > 1.0);
    assert_native_equal_flex_budget::<f64>(
        9,
        feasible,
        feasible_sum,
        &[0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    );
}
