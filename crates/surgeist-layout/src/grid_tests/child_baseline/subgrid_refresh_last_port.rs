// Intended child module of grid_tests/child_baseline_tests.rs.
// Native metrics are supplied only for leaves2/3/4. Both grids
// compute their own output/ports through the maintained production algorithm.
use super::*;
use crate::test_support::layout_tree::OracleTreeOf;

fn value<S: LayoutScalar>(n: f64) -> S {
    S::from_f64(n)
}

#[derive(Clone, Copy, Debug)]
enum FinalLeaf {
    Orthogonal,
    ParallelZero,
}

fn native_leaf<S: LayoutScalar>(
    size: Size<S>,
    first: Point<Option<S>>,
    last: Point<Option<S>>,
) -> ComputeOutputOf<S> {
    ComputeOutputOf::from_sizes_and_baselines(size, size, BaselinesOf { first, last })
}

fn tree<S: LayoutScalar>(
    inherited_columns: bool,
    first: f64,
    final_leaf: FinalLeaf,
) -> OracleTreeOf<S> {
    let p_columns = if inherited_columns {
        vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))]
    } else {
        vec![TrackComponentOf::px(value::<S>(20.0))]
    };
    let final_mode = match final_leaf {
        FinalLeaf::Orthogonal => WritingMode::VerticalRl,
        FinalLeaf::ParallelZero => WritingMode::HorizontalTb,
    };
    let final_metrics = match final_leaf {
        FinalLeaf::Orthogonal => native_leaf(
            Size::new(value::<S>(20.0), value::<S>(10.0)),
            Point::new(Some(value::<S>(3.0)), None),
            Point::new(Some(value::<S>(5.0)), None),
        ),
        FinalLeaf::ParallelZero => native_leaf(
            Size::new(value::<S>(20.0), value::<S>(10.0)),
            Point::new(None, Some(value::<S>(3.0))),
            Point::new(None, Some(S::ZERO)),
        ),
    };
    OracleTreeOf::new()
        .children(0, [1, 2])
        .children(1, [3, 4])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(value::<S>(40.0)),
                    PreferredSizeOf::px(value::<S>(40.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(value::<S>(20.0)); 2],
                grid_template_rows: vec![TrackComponentOf::px(value::<S>(40.0))],
                align_items: Some(AlignItems::LastBaseline),
                justify_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(value::<S>(20.0)),
                    PreferredSizeOf::px(value::<S>(20.0)),
                ),
                grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                grid_template_columns: p_columns,
                grid_template_rows: vec![TrackComponentOf::px(value::<S>(10.0)); 2],
                align_items: Some(AlignItems::Start),
                justify_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(value::<S>(20.0)),
                    PreferredSizeOf::px(value::<S>(20.0)),
                ),
                grid_column: GridPlacement::try_lines(2, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                size: Size::new(
                    PreferredSizeOf::px(value::<S>(20.0)),
                    PreferredSizeOf::px(value::<S>(10.0)),
                ),
                grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                writing_mode: final_mode,
                size: Size::new(
                    PreferredSizeOf::px(value::<S>(20.0)),
                    PreferredSizeOf::px(value::<S>(10.0)),
                ),
                grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                grid_row: GridPlacement::try_lines(2, 3).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .measure(
            2,
            native_leaf(
                Size::new(value::<S>(20.0), value::<S>(20.0)),
                Point::new(None, Some(value::<S>(13.0))),
                Point::new(None, Some(value::<S>(13.0))),
            ),
        )
        .measure(
            3,
            native_leaf(
                Size::new(value::<S>(20.0), value::<S>(10.0)),
                Point::new(None, Some(value::<S>(first))),
                Point::new(None, Some(value::<S>(first))),
            ),
        )
        .measure(4, final_metrics)
}

fn query<S: LayoutScalar>() -> ComputeInputOf<S> {
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
    )
}

fn assert_case<S: LayoutScalar>(first: f64, final_leaf: FinalLeaf) {
    let h = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    // Real standalone Grid control with equivalent fixed20 column: no measured
    // response for P. This qualifies production Grid's cross-axis port export.
    // It is not claimed to observe the later inherited-context invocation.
    let mut control = tree::<S>(false, first, final_leaf);
    let p = crate::compute_grid(&mut control.borrow_compute(), 1, query()).unwrap();
    assert_eq!(p.size, Size::new(value::<S>(20.0), value::<S>(20.0)));
    assert_eq!(
        p.baselines().first,
        Point::new(None, Some(value::<S>(first)))
    );
    match final_leaf {
        FinalLeaf::Orthogonal => {
            assert_eq!(p.baselines().last, Point::new(Some(value::<S>(5.0)), None));
            assert_eq!(p.baselines().last_block(h), None);
        }
        FinalLeaf::ParallelZero => {
            // Supplied leaf Last0 at second-row origin10 is P.LastY10.
            assert_eq!(p.baselines().last, Point::new(None, Some(value::<S>(10.0))));
        }
    }
    assert_eq!(control.layout(3).unwrap().location, Point::ZERO);
    assert_eq!(
        control.layout(4).unwrap().location,
        Point::new(S::ZERO, value::<S>(10.0))
    );

    let mut actual = tree::<S>(true, first, final_leaf);
    let input = query();
    let mut compute = actual.borrow_compute();
    let invocation = crate::grid::GridInvocation {
        container: crate::grid::input::GridContainerInput::from_node(Compute::node_input(
            &compute, 0,
        )),
        boundary: crate::grid::GridMeasurementBoundary::Ordinary,
    };
    let result = crate::grid::compute_grid_with_scrollbar_settlement(
        &mut compute,
        0,
        input,
        input.settled_auto_scrollbars(),
        |_| Ok(crate::grid::GridParentContext::none()),
        invocation,
        crate::grid::GridSizingStage::COMPLETE,
    )
    .unwrap();
    // The inherited axis is columns; rows are fixed40 and10/10, so the missing
    // Y Last has no intrinsic-row synthesis cycle to exclude this participant.
    let (p_y, anchor_y, under, last) = match final_leaf {
        FinalLeaf::Orthogonal => (13.0, 20.0, 7.0, 33.0),
        FinalLeaf::ParallelZero => (20.0, 17.0, 10.0, 30.0),
    };
    assert_eq!(
        actual.layout(1).unwrap().size,
        Size::new(value::<S>(20.0), value::<S>(20.0))
    );
    assert_eq!(
        actual.layout(1).unwrap().location,
        Point::new(S::ZERO, value::<S>(p_y))
    );
    assert_eq!(
        actual.layout(2).unwrap().location,
        Point::new(value::<S>(20.0), value::<S>(anchor_y))
    );
    assert_eq!(actual.layout(3).unwrap().location, Point::ZERO);
    assert_eq!(
        actual.layout(4).unwrap().location,
        Point::new(S::ZERO, value::<S>(10.0))
    );
    assert_eq!(
        result.output.size,
        Size::new(value::<S>(40.0), value::<S>(40.0))
    );
    // Export uses the earlier placement groups; preserve it independently of
    // the newly refreshed published groups. Do not predict an export regression.
    assert_eq!(
        result.output.baselines().first_block(h),
        Some(value::<S>(p_y + first))
    );
    assert_eq!(
        result.output.baselines().last_block(h),
        Some(value::<S>(last))
    );
    assert_eq!(result.baseline_groups.rows.len(), 1);
    assert!(result.baseline_groups.rows[0].first.is_none());
    // Intended missing-Last RED: synthesize20 =>under0, anchor13=>under7,
    // so max=7. Borrowed P.First7 incorrectly contributes under13 instead.
    assert_eq!(
        result.baseline_groups.rows[0]
            .last
            .unwrap()
            .coordinate_on(PhysicalAxis::Vertical),
        Some(value::<S>(under)),
        "{final_leaf:?}: refreshed Last group must not borrow First{first}"
    );
}

#[test]
fn subgrid_refresh_last_missing_does_not_borrow_first_f32() {
    for first in [7.0, 9.0] {
        assert_case::<f32>(first, FinalLeaf::Orthogonal);
    }
}
#[test]
fn subgrid_refresh_last_missing_does_not_borrow_first_f64() {
    for first in [7.0, 9.0] {
        assert_case::<f64>(first, FinalLeaf::Orthogonal);
    }
}
#[test]
fn subgrid_refresh_last_supplied_zero_stays_authoritative_f32() {
    assert_case::<f32>(7.0, FinalLeaf::ParallelZero);
}
#[test]
fn subgrid_refresh_last_supplied_zero_stays_authoritative_f64() {
    assert_case::<f64>(7.0, FinalLeaf::ParallelZero);
}
