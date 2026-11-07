use super::*;
use crate::test_support::readback::{self, Phase};

fn membership_leaf<S: LayoutScalar>(
    column: GridPlacement,
    width: f64,
    height: f64,
    alignment: AlignItems,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(S::from_f64(width), S::from_f64(height)).map(PreferredSizeOf::px),
        grid_column: column,
        justify_self: Some(AlignItems::Start),
        align_self: Some(alignment),
        ..NodeInputOf::default()
    }
}

fn dense_membership_parent<S: LayoutScalar>(height: f64) -> NodeInputOf<S> {
    let px = S::from_f64;
    NodeInputOf {
        display: Display::GridLanes,
        size: Size::new(px(30.0), px(height)).map(PreferredSizeOf::px),
        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, true),
        grid_template_columns: vec![TrackComponentOf::px(px(10.0)); 3],
        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
        gap: Size::new(LengthOf::ZERO, LengthOf::px(px(5.0))),
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    }
}

fn check_membership_boxes<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    expected: &[(u32, f64, f64, f64, f64)],
    label: &str,
    differences: &mut Vec<String>,
) {
    for (phase, entries) in [
        (
            "unrounded",
            readback::continuous_custody(batch, Phase::Unrounded).as_slice(),
        ),
        (
            "final",
            readback::continuous_custody(batch, Phase::Final).as_slice(),
        ),
    ] {
        for &(node, x, y, width, height) in expected {
            let Some(entry) = entries.iter().find(|entry| entry.node() == node) else {
                differences.push(format!("{label}, {phase}: missing node {node}"));
                continue;
            };
            let output = entry.output();
            let actual = (output.location, output.size);
            let expected = (
                Point::new(S::from_f64(x), S::from_f64(y)),
                Size::new(S::from_f64(width), S::from_f64(height)),
            );
            if actual != expected {
                differences.push(format!(
                    "{label}, {phase}, node {node}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
}

fn assert_dense_packed_membership_selects_boundary_baselines<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (alignment, peer_height, actor_y, peer_y) in [
        (AlignItems::Baseline, 30.0, 10.0, 0.0),
        (AlignItems::LastBaseline, 10.0, 40.0, 55.0),
    ] {
        let mut actor = dense_membership_parent::<S>(60.0);
        actor.grid_column = GridPlacement::try_line(1).unwrap();
        actor.grid_row = GridPlacement::try_line(1).unwrap();
        actor.justify_self = Some(AlignItems::Start);
        actor.align_self = Some(alignment);
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 6])
            .children(1, [2, 3, 4, 5])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(px(40.0), px(100.0)).map(PreferredSizeOf::px),
                    grid_template_columns: [30.0, 10.0]
                        .map(|extent| TrackComponentOf::px(px(extent)))
                        .into(),
                    grid_template_rows: vec![TrackComponentOf::px(px(100.0))],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(1, actor);
        for (node, column, width, height, margin_top) in [
            (2, GridPlacement::try_line(1).unwrap(), 10.0, 30.0, 0.0),
            (3, GridPlacement::try_lines(1, 3).unwrap(), 20.0, 10.0, 0.0),
            (4, GridPlacement::AUTO, 10.0, 20.0, 0.0),
            (5, GridPlacement::try_lines(1, 3).unwrap(), 20.0, 5.0, -30.0),
        ] {
            let mut child = membership_leaf::<S>(column, width, height, AlignItems::Start);
            child.margin.top = LengthAutoOf::px(px(margin_top));
            tree = tree
                .children(node, [])
                .style(node, child)
                // Size-only host facts leave physical border-baseline synthesis
                // to Layout. No test injects a private or named baseline.
                .measure(node, Size::new(px(width), px(height)));
        }
        let mut peer = membership_leaf::<S>(
            GridPlacement::try_line(2).unwrap(),
            10.0,
            peer_height,
            alignment,
        );
        peer.grid_row = GridPlacement::try_line(1).unwrap();
        tree = tree
            .children(6, [])
            .style(6, peer)
            .measure(6, Size::new(px(10.0), px(peer_height)));
        let label = format!("dense packed boundary {alignment:?}");
        let batch = match compute_layout(
            &tree,
            0,
            super::super::fixtures::fri08_c01_placement_request(),
        ) {
            Ok(batch) => batch,
            Err(error) => {
                differences.push(format!("{label}: {error:?}"));
                continue;
            }
        };
        // Jan Grid 3:1322 and Oct:816-818 select actual packed first/last
        // members, including dense backfill, independently of visual movement.
        // A occupies lane 1 at 0..30. B spans lanes 1/2 at 35..45. C backfills
        // lane 2 at 0..20; the third lane remains empty. D is packed last in
        // both occupied lanes at outer start50, with signed outer size5-30=-25.
        // Its border lies at20..25, visually before B, without changing membership.
        // Thus First=min(A30,C20)=20 and Last=D25; interior B45 is excluded.
        // First with peer30 gives target30, actor10 and peer0.
        // Last under-distances are actor60-25=35 and peer0: target100-35=65,
        // actor65-25=40 and peer65-10=55. The actor's own size remains30x60.
        check_membership_boxes(
            &batch,
            &[
                (0, 0.0, 0.0, 40.0, 100.0),
                (1, 0.0, actor_y, 30.0, 60.0),
                (2, 0.0, 0.0, 10.0, 30.0),
                (3, 0.0, 35.0, 20.0, 10.0),
                (4, 10.0, 0.0, 10.0, 20.0),
                (5, 0.0, 20.0, 20.0, 5.0),
                (6, 30.0, peer_y, 10.0, peer_height),
            ],
            &label,
            &mut differences,
        );
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn dense_packed_membership_selects_boundary_baselines_f32() {
    assert_dense_packed_membership_selects_boundary_baselines::<f32>();
}

#[test]
fn dense_packed_membership_selects_boundary_baselines_f64() {
    assert_dense_packed_membership_selects_boundary_baselines::<f64>();
}

fn assert_dense_backfill_uses_packed_following_spanner<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 3, 4])
        .style(1, dense_membership_parent::<S>(100.0));
    for (node, column, width, height, alignment) in [
        (
            2,
            GridPlacement::try_line(1).unwrap(),
            10.0,
            30.0,
            AlignItems::Start,
        ),
        (
            3,
            GridPlacement::try_lines(1, 3).unwrap(),
            20.0,
            10.0,
            AlignItems::Start,
        ),
        (4, GridPlacement::AUTO, 10.0, 20.0, AlignItems::End),
    ] {
        tree = tree
            .children(node, [])
            .style(node, membership_leaf::<S>(column, width, height, alignment))
            .measure(node, Size::new(px(width), px(height)));
    }
    let batch = compute_layout(
        &tree,
        1,
        super::super::fixtures::fri08_c01_placement_request(),
    )
    .unwrap();
    let mut differences = Vec::new();
    // Oct Grid 3:787-810 freezes each alignment container after placement.
    // C backfills lane 2 before B despite following B in source order. Its
    // fixed container ends at B's outer start35 minus gap5 = 30. End puts
    // C's20-unit box at10, not at the global terminal edge45 minus20 = 25.
    // A and B retain their original positions and sizes.
    check_membership_boxes(
        &batch,
        &[
            (1, 0.0, 0.0, 30.0, 100.0),
            (2, 0.0, 0.0, 10.0, 30.0),
            (3, 0.0, 35.0, 20.0, 10.0),
            (4, 10.0, 10.0, 10.0, 20.0),
        ],
        "dense packed following spanner",
        &mut differences,
    );
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn dense_backfill_uses_packed_following_spanner_f32() {
    assert_dense_backfill_uses_packed_following_spanner::<f32>();
}

#[test]
fn dense_backfill_uses_packed_following_spanner_f64() {
    assert_dense_backfill_uses_packed_following_spanner::<f64>();
}

fn assert_vertical_rtl_self_alignment_uses_logical_stacking_bounds<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut parent = NodeInputOf {
        display: Display::GridLanes,
        writing_mode: WritingMode::VerticalRl,
        direction: Direction::Rtl,
        size: Size::new(px(100.0), px(20.0)).map(PreferredSizeOf::px),
        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
        grid_template_columns: vec![TrackComponentOf::px(px(10.0)); 2],
        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    };
    // Explicitly zero gaps keep the independent physical table integral.
    parent.gap = Size::splat(LengthOf::ZERO);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2, 3])
        .style(1, parent);
    for (node, column, width, alignment) in [
        (2, 1, 20.0, AlignItems::End),
        (3, 2, 40.0, AlignItems::Start),
    ] {
        let mut child = membership_leaf::<S>(
            GridPlacement::try_line(column).unwrap(),
            width,
            10.0,
            alignment,
        );
        child.writing_mode = WritingMode::VerticalRl;
        child.direction = Direction::Rtl;
        tree = tree
            .children(node, [])
            .style(node, child)
            .measure(node, Size::new(px(width), px(10.0)));
    }
    let batch = compute_layout(
        &tree,
        1,
        super::super::fixtures::fri08_c01_placement_request(),
    )
    .unwrap();
    let mut differences = Vec::new();
    // The global packed terminal edge is logical block40. The short item's
    // local End offset is40-20=20. VerticalRl projects block start from the
    // physical right: x=100-20-20=60; the tall item is100-0-40=60.
    // RTL reverses the inline grid axis: lane1 y=20-0-10=10, lane2 y=0.
    check_membership_boxes(
        &batch,
        &[
            (1, 0.0, 0.0, 100.0, 20.0),
            (2, 60.0, 10.0, 20.0, 10.0),
            (3, 60.0, 0.0, 40.0, 10.0),
        ],
        "VerticalRl RTL logical stacking bounds",
        &mut differences,
    );
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn vertical_rtl_self_alignment_uses_logical_stacking_bounds_f32() {
    assert_vertical_rtl_self_alignment_uses_logical_stacking_bounds::<f32>();
}

#[test]
fn vertical_rtl_self_alignment_uses_logical_stacking_bounds_f64() {
    assert_vertical_rtl_self_alignment_uses_logical_stacking_bounds::<f64>();
}
