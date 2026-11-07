use surgeist::css::CssParser;
use surgeist::css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssLonghandValueRef,
};
use surgeist::css::values::layout::{
    CssGridAutoFlow, CssGridAutoFlowAxis, CssGridTrackList, CssItemDirection,
};
use surgeist::layout::{
    GridAxisKind, GridFlowToleranceOf, LaneItemOf, LanePlacementInputOf, LayoutScalar, place_lanes,
    resolve_grid_lanes_flow,
};

#[test]
fn effective_controls_resolve_orientation_without_erasing_auto_or_dense() {
    use GridAxisKind::{Column, Row};
    let none = CssGridTrackList::none();
    let tracks = CssGridTrackList::try_subgrid(Vec::new()).unwrap();
    let templates = [
        (&none, &none, Column),
        (&none, &tracks, Row),
        (&tracks, &none, Column),
        (&tracks, &tracks, Column),
    ];
    let flows = [
        (CssGridAutoFlow::Normal, None, false),
        (CssGridAutoFlow::Dense, None, true),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, false),
            Some(Column),
            false,
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true),
            Some(Column),
            true,
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, false),
            Some(Row),
            false,
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true),
            Some(Row),
            true,
        ),
    ];
    let directions = [
        (CssItemDirection::Auto, None, false),
        (CssItemDirection::Row, Some(Column), false),
        (CssItemDirection::Column, Some(Row), false),
        (CssItemDirection::RowReverse, Some(Column), true),
        (CssItemDirection::ColumnReverse, Some(Row), true),
    ];
    for (columns, rows, default_axis) in templates {
        for (auto_flow, explicit_axis, dense) in flows {
            for (direction, item_axis, reversed) in directions {
                let resolved =
                    resolve_grid_lanes_flow(direction, auto_flow, columns, rows).unwrap();
                assert_eq!(
                    resolved.grid_axis(),
                    item_axis.or(explicit_axis).unwrap_or(default_axis)
                );
                assert_eq!(resolved.reversed(), reversed);
                assert_eq!(resolved.is_dense(), dense);
                assert_ne!(resolved.stacking_axis(), resolved.grid_axis());
            }
        }
    }
}

fn independent_controls(text: &str) -> (CssItemDirection, CssGridAutoFlow) {
    let report = CssParser::default().parse_block_contents(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let mut direction = CssItemDirection::Auto;
    let mut auto_flow = CssGridAutoFlow::Normal;
    for declaration in report.syntax().iter() {
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expand().unwrap()
        else {
            panic!("ordinary longhand contributions");
        };
        for value in values.items() {
            match value.value() {
                CssContributionValueRef::Ordinary(CssLonghandValueRef::ItemDirection(value)) => {
                    direction = *value
                }
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridAutoFlow(value)) => {
                    auto_flow = *value
                }
                _ => {}
            }
        }
    }
    (direction, auto_flow)
}

#[test]
fn distinct_controls_and_shorthand_direction_are_independent_of_declaration_order() {
    let none = CssGridTrackList::none();
    let rows = CssGridTrackList::try_subgrid(Vec::new()).unwrap();
    for (direction, expected_axis, reversed) in [
        ("row", GridAxisKind::Column, false),
        ("column-reverse", GridAxisKind::Row, true),
        ("auto", GridAxisKind::Row, false),
    ] {
        for property in ["item-direction", "item-flow"] {
            for declarations in [
                format!("{property}:{direction};grid-auto-flow:dense"),
                format!("grid-auto-flow:dense;{property}:{direction}"),
            ] {
                let (direction, auto_flow) = independent_controls(&declarations);
                let resolved = resolve_grid_lanes_flow(direction, auto_flow, &none, &rows).unwrap();
                assert_eq!(resolved.grid_axis(), expected_axis, "{declarations}");
                assert_eq!(resolved.reversed(), reversed, "{declarations}");
                assert!(resolved.is_dense(), "{declarations}");
            }
        }
    }
}

fn assert_css_reverse_reaches_lane_placement<S: LayoutScalar>() {
    let none = CssGridTrackList::none();
    for direction in [
        CssItemDirection::RowReverse,
        CssItemDirection::ColumnReverse,
    ] {
        let flow =
            resolve_grid_lanes_flow(direction, CssGridAutoFlow::Normal, &none, &none).unwrap();
        let report = place_lanes(LanePlacementInputOf {
            grid_axis_tracks: 3,
            grid_axis: flow.grid_axis(),
            reversed: flow.reversed(),
            lane_gap: S::ZERO,
            tolerance: GridFlowToleranceOf::Infinite,
            tolerance_basis: S::ZERO,
            items: (0..4)
                .map(|item| LaneItemOf {
                    item,
                    grid_axis_span: 1,
                    definite_grid_axis_start: None,
                    lane_axis_margin_box: S::from_usize(10),
                })
                .collect(),
        })
        .unwrap();
        assert_eq!(report.grid_axis, flow.grid_axis());
        assert_eq!(report.lane_axis, flow.stacking_axis());
        assert_eq!(
            report
                .item_offsets
                .iter()
                .map(|item| item.grid_axis_start)
                .collect::<Vec<_>>(),
            [3, 2, 1, 3]
        );
        assert_eq!(
            report
                .item_offsets
                .iter()
                .map(|item| item.offset)
                .collect::<Vec<_>>(),
            [S::ZERO, S::ZERO, S::ZERO, S::from_usize(10)]
        );
    }
}

#[test]
fn css_reverse_reaches_lane_placement_f32() {
    assert_css_reverse_reaches_lane_placement::<f32>();
}

#[test]
fn css_reverse_reaches_lane_placement_f64() {
    assert_css_reverse_reaches_lane_placement::<f64>();
}
