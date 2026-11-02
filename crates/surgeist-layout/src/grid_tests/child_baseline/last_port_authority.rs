// Intended child of grid_tests/child_baseline_tests.rs. This exercises actual
// Grid consumers of native metric outputs, not just BaselinesOf's helper.
use super::*;
use crate::engine::contracts::Compute;
use crate::test_support::layout_tree::OracleTreeOf;

#[derive(Clone, Copy, Debug)]
enum LastPort {
    Missing,
    Zero,
    Thirteen,
}
impl LastPort {
    fn supplied(self) -> Option<f64> {
        match self {
            Self::Missing => None,
            Self::Zero => Some(0.0),
            Self::Thirteen => Some(13.0),
        }
    }
}
fn s<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn baseline_point<S: LayoutScalar>(axes: FlowAxes, value: Option<f64>) -> Point<Option<S>> {
    match axes.block_axis() {
        PhysicalAxis::Horizontal => Point::new(value.map(s::<S>), None),
        PhysicalAxis::Vertical => Point::new(None, value.map(s::<S>)),
    }
}
fn metric_leaf<S: LayoutScalar>(
    axes: FlowAxes,
    first: f64,
    last: Option<f64>,
) -> ComputeOutputOf<S> {
    let first = baseline_point(axes, Some(first));
    let baselines = match last {
        None => BaselinesOf::first(first),
        Some(last) => BaselinesOf {
            first,
            last: baseline_point(axes, Some(last)),
        },
    };
    ComputeOutputOf::from_sizes_and_baselines(
        Size::new(s::<S>(20.0), s::<S>(20.0)),
        Size::new(s::<S>(20.0), s::<S>(20.0)),
        baselines,
    )
}
fn metric_tree<S: LayoutScalar>(
    port: LastPort,
    direction: Direction,
    alignment: AlignItems,
    subject_mode: WritingMode,
    spans_auto_rows: bool,
) -> OracleTreeOf<S> {
    let parent_axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
    let subject_axes = FlowAxes::new(subject_mode, direction);
    let mut subject = NodeInputOf {
        display: Display::Block,
        writing_mode: subject_mode,
        direction,
        size: Size::new(
            PreferredSizeOf::px(s::<S>(20.0)),
            PreferredSizeOf::px(s::<S>(20.0)),
        ),
        ..NodeInputOf::default()
    };
    let rows = if spans_auto_rows {
        subject.grid_row = GridPlacement::try_lines(1, 3).unwrap();
        vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO]
    } else {
        vec![TrackComponentOf::px(s::<S>(40.0))]
    };
    OracleTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(s::<S>(40.0)),
                    PreferredSizeOf::px(s::<S>(40.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(s::<S>(20.0)); 2],
                grid_template_rows: rows,
                align_items: Some(alignment),
                ..NodeInputOf::default()
            },
        )
        .style(1, subject)
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(s::<S>(20.0)),
                    PreferredSizeOf::px(s::<S>(20.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        // Faithful native metric leaves: the maintained owner returns these
        // declared physical metrics through Compute::compute_child. No Text,
        // font/named-baseline/capture authority or alternate Grid is invented.
        .measure(1, metric_leaf(subject_axes, 7.0, port.supplied()))
        .measure(2, metric_leaf(parent_axes, 13.0, Some(13.0)))
}
fn performed_leaf<S: LayoutScalar>(
    tree: &mut OracleTreeOf<S>,
    node: u32,
    direction: Direction,
) -> ComputeOutputOf<S> {
    let mut compute = tree.borrow_compute();
    compute
        .compute_child(
            node,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::new(Some(s::<S>(20.0)), Some(s::<S>(20.0))),
                Size::new(Some(s::<S>(20.0)), Some(s::<S>(40.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, direction),
                    ParentFormattingContext::Grid,
                ),
                Size::new(
                    AvailableOf::definite(s::<S>(20.0)),
                    AvailableOf::definite(s::<S>(40.0)),
                ),
            ),
        )
        .unwrap()
}
fn root_query<S: LayoutScalar>(direction: Direction) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, direction),
            ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    )
}
fn qualify<S: LayoutScalar>(output: ComputeOutputOf<S>, port: LastPort, axes: FlowAxes) {
    assert_eq!(output.size, Size::new(s::<S>(20.0), s::<S>(20.0)));
    assert_eq!(output.baselines().first_block(axes), Some(s::<S>(7.0)));
    assert_eq!(
        output.baselines().last_block(axes),
        port.supplied().map(s::<S>)
    );
    assert_eq!(output.named_baselines, NamedBaselinesOf::unknown());
}
fn placement<S: LayoutScalar>(port: LastPort, alignment: AlignItems) {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
        let mut tree =
            metric_tree::<S>(port, direction, alignment, WritingMode::HorizontalTb, false);
        let measured = performed_leaf(&mut tree, 1, direction);
        qualify(measured, port, axes);
        let input = root_query(direction);
        let result = crate::grid::compute_grid_with_scrollbar_settlement(
            &mut tree.borrow_compute(),
            0,
            input,
            input.settled_auto_scrollbars(),
            |_| Ok(crate::grid::GridParentContext::none()),
            crate::grid::GridMeasurementBoundary::Ordinary,
            crate::grid::GridSizingStage::COMPLETE,
        )
        .unwrap();
        // This is the real published group retained for downstream baseline
        // consumers, built by child/mod.rs from its stored selected Last port.
        // Its minor coordinate is distance from the under edge, not global Y.
        assert_eq!(result.baseline_groups.rows.len(), 1);
        let row = result.baseline_groups.rows[0];
        match alignment {
            AlignItems::LastBaseline => {
                assert!(row.first.is_none());
                let under = match port {
                    LastPort::Missing => 7.0,
                    LastPort::Zero => 20.0,
                    LastPort::Thirteen => 7.0,
                };
                assert_eq!(
                    row.last.unwrap().coordinate_on(PhysicalAxis::Vertical),
                    Some(s::<S>(under)),
                    "{port:?}: actual published minor group must use its selected Last ports"
                );
            }
            AlignItems::Baseline => {
                assert!(row.last.is_none());
                assert_eq!(
                    row.first.unwrap().coordinate_on(PhysicalAxis::Vertical),
                    Some(s::<S>(13.0))
                );
            }
            _ => unreachable!("the qualifier chooses a real baseline alignment"),
        }
        let output = result.output;
        // Last:20px boxes have under-distances0/7 (missing),20/7 (zero),7/7 (13).
        // Anchor the joint under-distance at row-end40; never borrow First7.
        let (subject_y, anchor_y, first, last) = match alignment {
            AlignItems::LastBaseline => match port {
                LastPort::Missing => (13.0, 20.0, 20.0, 33.0),
                LastPort::Zero => (20.0, 7.0, 27.0, 20.0),
                LastPort::Thirteen => (20.0, 20.0, 27.0, 33.0),
            },
            AlignItems::Baseline => (6.0, 0.0, 13.0, 13.0),
            _ => unreachable!("the qualifier chooses a real baseline alignment"),
        };
        let (subject_x, anchor_x) = if direction == Direction::Rtl {
            (20.0, 0.0)
        } else {
            (0.0, 20.0)
        };
        for (id, x, y) in [(1, subject_x, subject_y), (2, anchor_x, anchor_y)] {
            let actual = tree.layout(id).unwrap();
            assert_eq!(
                actual.location,
                Point::new(s::<S>(x), s::<S>(y)),
                "{port:?} {alignment:?} {direction:?} node={id}"
            );
            assert_eq!(actual.size, Size::new(s::<S>(20.0), s::<S>(20.0)));
            assert_eq!(
                (actual.margin, actual.padding, actual.border),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO)
            );
            let geometry = actual.scroll_geometry.unwrap();
            assert_eq!(geometry.border_box().origin(), Point::ZERO);
            assert_eq!(geometry.border_box().size(), actual.size);
            assert_eq!(geometry.scrollable_overflow(), geometry.border_box());
        }
        assert_eq!(output.size, Size::new(s::<S>(40.0), s::<S>(40.0)));
        assert_eq!(output.content_size, Size::new(s::<S>(40.0), s::<S>(40.0)));
        assert_eq!(
            output.baselines(),
            BaselinesOf {
                first: Point::new(None, Some(s::<S>(first))),
                last: Point::new(None, Some(s::<S>(last))),
            }
        );
    }
}
fn membership<S: LayoutScalar>(
    output: ComputeOutputOf<S>,
    alignment: AlignItems,
    child_axes: FlowAxes,
    containing_axes: FlowAxes,
    span: GridTrackSpan,
    cycle: bool,
) -> Option<AncestorBaselineMember<u32, S>> {
    ancestor_baseline_member(AncestorBaselineMemberInput {
        source: 1,
        axis: GridAxisKind::Row,
        ancestor_span: span,
        alignment_context_reversed: false,
        alignment,
        block_auto_margins: false,
        synthesized_baseline_cycle: cycle,
        output,
        margin: Edges::ZERO,
        child_flow_axes: child_axes,
        containing_flow_axes: containing_axes,
        start_adjustment: S::ZERO,
        end_adjustment: S::ZERO,
    })
}
fn intrinsic_member<S: LayoutScalar>(port: LastPort) {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = metric_tree::<S>(
        port,
        Direction::Ltr,
        AlignItems::LastBaseline,
        WritingMode::HorizontalTb,
        false,
    );
    let measured = performed_leaf(&mut tree, 1, Direction::Ltr);
    qualify(measured, port, axes);
    let span = GridTrackSpan::new(1, 2); // actual sole declared40px row
    let subject = membership(measured, AlignItems::LastBaseline, axes, axes, span, false).unwrap();
    assert_eq!(subject.role(), AncestorBaselineRole::Last);
    assert_eq!(subject.selected_track(), 0);
    let solo =
        AncestorBaselineGroup::reduce(0, GridAxisKind::Row, PhysicalAxis::Vertical, 1, [subject]);
    let expected_under = match port {
        LastPort::Missing => 0.0,
        LastPort::Zero => 20.0,
        LastPort::Thirteen => 7.0,
    };
    assert_eq!(
        solo.target_for(subject),
        Some(s::<S>(expected_under)),
        "{port:?}: actual intrinsic member's line-under distance"
    );
    let anchor_output = performed_leaf(&mut tree, 2, Direction::Ltr);
    let anchor = membership(
        anchor_output,
        AlignItems::LastBaseline,
        axes,
        axes,
        span,
        false,
    )
    .unwrap();
    let group = AncestorBaselineGroup::reduce(
        0,
        GridAxisKind::Row,
        PhysicalAxis::Vertical,
        1,
        [subject, anchor],
    );
    let (subject_y, anchor_y) = match port {
        LastPort::Missing => (13.0, 20.0),
        LastPort::Zero => (20.0, 7.0),
        LastPort::Thirteen => (20.0, 20.0),
    };
    assert_eq!(
        group.placement_offset(subject, S::ZERO, s::<S>(40.0)),
        Some(s::<S>(subject_y))
    );
    assert_eq!(
        group.placement_offset(anchor, S::ZERO, s::<S>(40.0)),
        Some(s::<S>(anchor_y))
    );
    let first = membership(measured, AlignItems::Baseline, axes, axes, span, false).unwrap();
    assert_eq!(first.role(), AncestorBaselineRole::First);
    let first_group =
        AncestorBaselineGroup::reduce(0, GridAxisKind::Row, PhysicalAxis::Vertical, 1, [first]);
    assert_eq!(
        first_group.target_for(first),
        Some(s::<S>(7.0)),
        "Last absence or zero must not replace genuine First7"
    );
}
fn axis_and_cycle<S: LayoutScalar>() {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for port in [LastPort::Missing, LastPort::Zero, LastPort::Thirteen] {
        // A genuine declared two-auto-row span, with cycle classification from
        // the real selected-set facts and track kinds, not a fabricated flag.
        let mut tree = metric_tree::<S>(
            port,
            Direction::Ltr,
            AlignItems::LastBaseline,
            WritingMode::HorizontalTb,
            true,
        );
        let measured = performed_leaf(&mut tree, 1, Direction::Ltr);
        qualify(measured, port, axes);
        let rows = vec![TrackSizingOf::<S>::AUTO; 2];
        let cycle = synthesized_baseline_would_cycle(
            AlignItems::LastBaseline,
            measured.baselines(),
            axes,
            &rows,
            None,
        );
        assert_eq!(cycle, matches!(port, LastPort::Missing));
        let participation = baseline_participation(
            AlignItems::LastBaseline,
            false,
            cycle,
            measured.baselines(),
            axes,
        );
        assert_eq!(participation.synthesized, matches!(port, LastPort::Missing));
        let member = membership(
            measured,
            AlignItems::LastBaseline,
            axes,
            axes,
            GridTrackSpan::new(1, 3),
            cycle,
        );
        assert_eq!(member.is_none(), matches!(port, LastPort::Missing));
        let first_cycle = synthesized_baseline_would_cycle(
            AlignItems::Baseline,
            measured.baselines(),
            axes,
            &rows,
            None,
        );
        assert!(
            !first_cycle,
            "real First7 remains supplied even while Last is missing"
        );
        assert!(
            membership(
                measured,
                AlignItems::Baseline,
                axes,
                axes,
                GridTrackSpan::new(1, 3),
                first_cycle
            )
            .is_some()
        );

        // Orthogonal native metric leaf really supplies its metrics on X.
        // A horizontal Grid row requires Y and cannot borrow that authority.
        let child_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
        let mut tree = metric_tree::<S>(
            port,
            Direction::Ltr,
            AlignItems::LastBaseline,
            WritingMode::VerticalRl,
            false,
        );
        let orthogonal = performed_leaf(&mut tree, 1, Direction::Ltr);
        qualify(orthogonal, port, child_axes);
        assert!(
            membership(
                orthogonal,
                AlignItems::LastBaseline,
                child_axes,
                axes,
                GridTrackSpan::new(1, 2),
                false
            )
            .is_none()
        );
    }
}

macro_rules! port_cases {
    ($place:ident, $member:ident, $scalar:ty, $port:expr) => {
        #[test]
        fn $place() {
            placement::<$scalar>($port, AlignItems::LastBaseline);
        }
        #[test]
        fn $member() {
            intrinsic_member::<$scalar>($port);
        }
    };
}
port_cases!(
    grid_last_missing_does_not_borrow_first_f32,
    intrinsic_last_missing_does_not_borrow_first_f32,
    f32,
    LastPort::Missing
);
port_cases!(
    grid_last_zero_remains_supplied_f32,
    intrinsic_last_zero_remains_supplied_f32,
    f32,
    LastPort::Zero
);
port_cases!(
    grid_last_thirteen_remains_supplied_f32,
    intrinsic_last_thirteen_remains_supplied_f32,
    f32,
    LastPort::Thirteen
);
port_cases!(
    grid_last_missing_does_not_borrow_first_f64,
    intrinsic_last_missing_does_not_borrow_first_f64,
    f64,
    LastPort::Missing
);
port_cases!(
    grid_last_zero_remains_supplied_f64,
    intrinsic_last_zero_remains_supplied_f64,
    f64,
    LastPort::Zero
);
port_cases!(
    grid_last_thirteen_remains_supplied_f64,
    intrinsic_last_thirteen_remains_supplied_f64,
    f64,
    LastPort::Thirteen
);
#[test]
fn grid_first_port_stays_independent_f32() {
    for port in [LastPort::Missing, LastPort::Zero, LastPort::Thirteen] {
        placement::<f32>(port, AlignItems::Baseline);
    }
}
#[test]
fn grid_first_port_stays_independent_f64() {
    for port in [LastPort::Missing, LastPort::Zero, LastPort::Thirteen] {
        placement::<f64>(port, AlignItems::Baseline);
    }
}
#[test]
fn last_synthesis_keeps_axis_and_cycle_guards_f32() {
    axis_and_cycle::<f32>();
}
#[test]
fn last_synthesis_keeps_axis_and_cycle_guards_f64() {
    axis_and_cycle::<f64>();
}
