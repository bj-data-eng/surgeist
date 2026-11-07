use surgeist::css::CssParser;
use surgeist::css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssLonghandValueRef,
};
use surgeist::css::values::layout::{
    CssGridAutoFlow, CssGridAutoFlowAxis, CssGridTrackList, CssItemDirection, CssItemPack,
};
use surgeist::layout::{
    AlignContent, AvailableSpaceOf, AxisSizingRequestOf, Direction, Display, FlowAxes,
    GridAxisKind, GridFlowToleranceOf, GridLanesFlow, GridPlacement, LaneItemOf,
    LanePlacementInputOf, LayoutInputOf, LayoutInputViewOf, LayoutRootRequestOf, LayoutScalar,
    LayoutTree, LengthOf, LengthPercentageOf, NearestScrollportOf, NodeInputOf,
    NonNegativeFiniteOf, OrthogonalFallbackLimitOf, OrthogonalFallbackOf, Point, PreferredSizeOf,
    Size, SizingCalculationOf, SizingOperation, SizingRequestOf, TrackComponentOf, TrackSizingOf,
    Traverse, WritingMode, compute_layout, place_lanes, resolve_grid_lanes_flow,
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
                for (pack, pack_dense) in [
                    (CssItemPack::Normal, false),
                    (CssItemPack::Balance, false),
                    (CssItemPack::Dense, true),
                    (CssItemPack::DenseBalance, true),
                ] {
                    let resolved =
                        resolve_grid_lanes_flow(direction, auto_flow, pack, columns, rows).unwrap();
                    assert_eq!(
                        resolved.grid_axis(),
                        item_axis.or(explicit_axis).unwrap_or(default_axis)
                    );
                    assert_eq!(resolved.reversed(), reversed);
                    assert_eq!(resolved.is_dense(), dense || pack_dense);
                    assert_ne!(resolved.stacking_axis(), resolved.grid_axis());
                }
            }
        }
    }
}

fn independent_controls(text: &str) -> (CssItemDirection, CssGridAutoFlow, CssItemPack) {
    let report = CssParser::default().parse_block_contents(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let mut direction = CssItemDirection::Auto;
    let mut auto_flow = CssGridAutoFlow::Normal;
    let mut pack = CssItemPack::Normal;
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
                CssContributionValueRef::Ordinary(CssLonghandValueRef::ItemPack(value)) => {
                    pack = *value
                }
                _ => {}
            }
        }
    }
    (direction, auto_flow, pack)
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
                let (direction, auto_flow, pack) = independent_controls(&declarations);
                let resolved =
                    resolve_grid_lanes_flow(direction, auto_flow, pack, &none, &rows).unwrap();
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
        let flow = resolve_grid_lanes_flow(
            direction,
            CssGridAutoFlow::Normal,
            CssItemPack::Normal,
            &none,
            &none,
        )
        .unwrap();
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

struct BackfillTree<S: LayoutScalar> {
    inputs: Vec<LayoutInputOf<S>>,
}

impl<S: LayoutScalar> Traverse for BackfillTree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1, 2, 3, 4] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        if node == 0 { 4 } else { 0 }
    }

    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl<S: LayoutScalar> LayoutTree for BackfillTree<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn assert_backfill_geometry<S: LayoutScalar>(
    flow: GridLanesFlow,
    expected_axis: GridAxisKind,
    expected_dense: bool,
) {
    let px = S::from_f64;
    assert_eq!(flow.grid_axis(), expected_axis);
    assert!(!flow.reversed());
    let columns = expected_axis == GridAxisKind::Column;
    let physical_size = |grid, stacking| {
        if columns {
            Size::new(grid, stacking)
        } else {
            Size::new(stacking, grid)
        }
    };
    let root_size = physical_size(px(20.0), px(100.0));
    let preferred_size = |value| PreferredSizeOf::value(LengthPercentageOf::px(value).unwrap());
    let fixed_track = TrackComponentOf::from(TrackSizingOf::calculation(
        SizingCalculationOf::value(LengthPercentageOf::px(px(10.0)).unwrap()),
    ));
    let mut inputs = vec![LayoutInputOf::box_input(NodeInputOf {
        display: Display::GridLanes,
        grid_lanes_flow: flow,
        size: root_size.map(preferred_size),
        grid_template_columns: if columns {
            vec![fixed_track.clone(); 2]
        } else {
            Vec::new()
        },
        grid_template_rows: if columns {
            Vec::new()
        } else {
            vec![fixed_track.clone(); 2]
        },
        grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    })];
    for (placement, height) in [
        (GridPlacement::try_line(2).unwrap(), 40.0),
        (GridPlacement::try_lines(1, 3).unwrap(), 10.0),
        (GridPlacement::AUTO, 25.0),
        (GridPlacement::AUTO, 10.0),
    ] {
        inputs.push(LayoutInputOf::box_input(NodeInputOf {
            grid_column: if columns {
                placement
            } else {
                GridPlacement::AUTO
            },
            grid_row: if columns {
                GridPlacement::AUTO
            } else {
                placement
            },
            size: if columns {
                Size::new(PreferredSizeOf::AUTO, preferred_size(px(height)))
            } else {
                Size::new(preferred_size(px(height)), PreferredSizeOf::AUTO)
            },
            ..NodeInputOf::default()
        }));
    }
    let axes = root_size.map(|size| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(size).unwrap()),
            SizingOperation::Normal,
        )
    });
    let parent = axes.map(|axis| axis.space().definite_size());
    let batch = compute_layout(
        &BackfillTree { inputs },
        0,
        LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    parent,
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            parent,
        ),
    )
    .unwrap();
    // The definite 40-high item occupies track 2. The two-track bridge starts
    // at 40 and advances both tracks to 50, leaving a 0..40 hole in track 1.
    // Sparse items start at 50 in tracks 1 then 2. Dense items fit at 0 and 25
    // in track 1, with no change to any item's dimensions or the grid tracks.
    for (node, grid, stacking, width, height) in [
        (0, 0.0, 0.0, 20.0, 100.0),
        (1, 10.0, 0.0, 10.0, 40.0),
        (2, 0.0, 40.0, 20.0, 10.0),
        (3, 0.0, if expected_dense { 0.0 } else { 50.0 }, 10.0, 25.0),
        (
            4,
            if expected_dense { 0.0 } else { 10.0 },
            if expected_dense { 25.0 } else { 50.0 },
            10.0,
            10.0,
        ),
    ] {
        let output = batch
            .node_layout(node)
            .unwrap()
            .single_box()
            .expect("fixture node produces one box")
            .unrounded_output();
        let location = if columns {
            Point::new(px(grid), px(stacking))
        } else {
            Point::new(px(stacking), px(grid))
        };
        assert_eq!(
            (output.location, output.size),
            (location, physical_size(px(width), px(height))),
            "{flow:?}, node {node}"
        );
    }
}

fn assert_effective_pack_backfill<S: LayoutScalar>() {
    let none = CssGridTrackList::none();
    for (direction, opposite_auto_axis, expected_axis) in [
        (
            CssItemDirection::Row,
            CssGridAutoFlowAxis::Column,
            GridAxisKind::Column,
        ),
        (
            CssItemDirection::Column,
            CssGridAutoFlowAxis::Row,
            GridAxisKind::Row,
        ),
    ] {
        for auto_dense in [false, true] {
            let auto_flow = CssGridAutoFlow::explicit_axis(opposite_auto_axis, auto_dense);
            for (pack, expected_dense) in [
                (CssItemPack::Normal, auto_dense),
                (CssItemPack::Balance, auto_dense),
                (CssItemPack::Dense, true),
                (CssItemPack::DenseBalance, true),
            ] {
                let flow =
                    resolve_grid_lanes_flow(direction, auto_flow, pack, &none, &none).unwrap();
                assert_backfill_geometry::<S>(flow, expected_axis, expected_dense);
            }
        }
    }
}

#[test]
fn effective_item_pack_reaches_sparse_and_dense_backfill_f32() {
    assert_effective_pack_backfill::<f32>();
}

#[test]
fn effective_item_pack_reaches_sparse_and_dense_backfill_f64() {
    assert_effective_pack_backfill::<f64>();
}

#[test]
fn item_flow_pack_and_longhands_keep_effective_controls_across_declaration_order() {
    let none = CssGridTrackList::none();
    for (direction_text, direction, expected_axis) in [
        ("row", CssItemDirection::Row, GridAxisKind::Column),
        ("column", CssItemDirection::Column, GridAxisKind::Row),
    ] {
        for (flow_text, auto_flow, auto_dense) in [
            ("normal", CssGridAutoFlow::Normal, false),
            ("dense", CssGridAutoFlow::Dense, true),
            (
                "row",
                CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, false),
                false,
            ),
            (
                "column",
                CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, false),
                false,
            ),
            (
                "row dense",
                CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true),
                true,
            ),
            (
                "column dense",
                CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true),
                true,
            ),
        ] {
            for (pack_text, pack, expected_dense) in [
                ("normal", CssItemPack::Normal, auto_dense),
                ("balance", CssItemPack::Balance, auto_dense),
                ("dense", CssItemPack::Dense, true),
                ("dense balance", CssItemPack::DenseBalance, true),
            ] {
                for declarations in [
                    format!("item-flow:{direction_text} {pack_text};grid-auto-flow:{flow_text}"),
                    format!("grid-auto-flow:{flow_text};item-flow:{direction_text} {pack_text}"),
                    format!(
                        "item-direction:{direction_text};item-pack:{pack_text};grid-auto-flow:{flow_text}"
                    ),
                    format!(
                        "grid-auto-flow:{flow_text};item-pack:{pack_text};item-direction:{direction_text}"
                    ),
                ] {
                    let controls = independent_controls(&declarations);
                    assert_eq!(controls, (direction, auto_flow, pack), "{declarations}");
                    let resolved =
                        resolve_grid_lanes_flow(controls.0, controls.1, controls.2, &none, &none)
                            .unwrap();
                    assert_backfill_geometry::<f32>(resolved, expected_axis, expected_dense);
                }
            }
        }
    }
}
