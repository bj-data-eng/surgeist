use surgeist::css::CssParser;
use surgeist::css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssLonghandValueRef,
};
use surgeist::css::values::layout::{CssGridAutoFlow, CssGridAutoFlowAxis, CssItemPack};
use surgeist::layout::{
    AlignContent, AlignItems, AvailableSpaceOf, AxisSizingRequestOf, Direction, Display, FlowAxes,
    GridAutoFlow, GridPlacement, ItemOrder, LayoutInputOf, LayoutInputViewOf, LayoutRootRequestOf,
    LayoutScalar, LayoutTree, LengthPercentageOf, NearestScrollportOf, NodeInputOf,
    OrthogonalFallbackLimitOf, OrthogonalFallbackOf, Point, PreferredSizeOf, Size,
    SizingCalculationOf, SizingOperation, SizingRequestOf, TrackComponentOf, TrackSizingOf,
    Traverse, WritingMode, compute_layout, resolve_grid_auto_flow,
};

struct FlowCase {
    spelling: &'static str,
    authored: CssGridAutoFlow,
    normal_pack: GridAutoFlow,
    dense_pack: GridAutoFlow,
    column: bool,
}

fn flow_cases() -> [FlowCase; 6] {
    [
        FlowCase {
            spelling: "normal",
            authored: CssGridAutoFlow::Normal,
            normal_pack: GridAutoFlow::Row,
            dense_pack: GridAutoFlow::RowDense,
            column: false,
        },
        FlowCase {
            spelling: "dense",
            authored: CssGridAutoFlow::Dense,
            normal_pack: GridAutoFlow::RowDense,
            dense_pack: GridAutoFlow::RowDense,
            column: false,
        },
        FlowCase {
            spelling: "row",
            authored: CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, false),
            normal_pack: GridAutoFlow::Row,
            dense_pack: GridAutoFlow::RowDense,
            column: false,
        },
        FlowCase {
            spelling: "row dense",
            authored: CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true),
            normal_pack: GridAutoFlow::RowDense,
            dense_pack: GridAutoFlow::RowDense,
            column: false,
        },
        FlowCase {
            spelling: "column",
            authored: CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, false),
            normal_pack: GridAutoFlow::Column,
            dense_pack: GridAutoFlow::ColumnDense,
            column: true,
        },
        FlowCase {
            spelling: "column dense",
            authored: CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true),
            normal_pack: GridAutoFlow::ColumnDense,
            dense_pack: GridAutoFlow::ColumnDense,
            column: true,
        },
    ]
}

fn pack_cases() -> [(&'static str, CssItemPack, bool); 4] {
    [
        ("normal", CssItemPack::Normal, false),
        ("balance", CssItemPack::Balance, false),
        ("dense", CssItemPack::Dense, true),
        ("dense balance", CssItemPack::DenseBalance, true),
    ]
}

struct GridTree<S: LayoutScalar> {
    inputs: Vec<LayoutInputOf<S>>,
}

impl<S: LayoutScalar> Traverse for GridTree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1, 2, 3] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        if node == 0 { 3 } else { 0 }
    }

    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl<S: LayoutScalar> LayoutTree for GridTree<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn assert_grid_geometry<S: LayoutScalar>(flow: GridAutoFlow, column: bool, dense: bool) {
    let px = S::from_f64;
    let fixed_track = TrackComponentOf::from(TrackSizingOf::calculation(
        SizingCalculationOf::value(LengthPercentageOf::px(px(10.0)).unwrap()),
    ));
    let ten = PreferredSizeOf::value(LengthPercentageOf::px(px(10.0)).unwrap());
    for reordered in [false, true] {
        let mut inputs = vec![LayoutInputOf::box_input(NodeInputOf {
            display: Display::Grid,
            grid_auto_flow: flow,
            grid_template_columns: vec![fixed_track.clone(); if column { 1 } else { 3 }],
            grid_template_rows: vec![fixed_track.clone(); if column { 3 } else { 1 }],
            grid_auto_columns: vec![fixed_track.clone()],
            grid_auto_rows: vec![fixed_track.clone()],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            justify_items: Some(AlignItems::Stretch),
            align_items: Some(AlignItems::Stretch),
            ..NodeInputOf::default()
        })];
        let spans = if reordered {
            [(2, 0), (1, 2), (2, 1)]
        } else {
            [(2, 0), (2, 0), (1, 0)]
        };
        for (span, order) in spans {
            let placement = GridPlacement::try_span(span).unwrap();
            inputs.push(LayoutInputOf::box_input(NodeInputOf {
                item_order: ItemOrder::new(order),
                grid_column: if column {
                    GridPlacement::AUTO
                } else {
                    placement
                },
                grid_row: if column {
                    placement
                } else {
                    GridPlacement::AUTO
                },
                size: if column {
                    Size::new(ten.clone(), PreferredSizeOf::AUTO)
                } else {
                    Size::new(PreferredSizeOf::AUTO, ten.clone())
                },
                ..NodeInputOf::default()
            }));
        }
        let batch = compute_layout(
            &GridTree { inputs },
            0,
            LayoutRootRequestOf::standalone(
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
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::NONE,
            ),
        )
        .unwrap();
        // Two span-2 items cannot share the first three-track row. The second
        // creates a 10-unit implicit row. Sparse packing leaves its predecessor's
        // third cell empty; dense packing fills that cell with the span-1 item.
        // Column flow transposes these coordinates. Source order 2,1,2 with
        // orders 0,2,1 retains the same processing order while moving source IDs.
        let (second, last) = if reordered { (3, 2) } else { (2, 3) };
        for (node, x, y, width, height) in [
            (0, 0.0, 0.0, 30.0, 20.0),
            (1, 0.0, 0.0, 20.0, 10.0),
            (second, 0.0, 10.0, 20.0, 10.0),
            (last, 20.0, if dense { 0.0 } else { 10.0 }, 10.0, 10.0),
        ] {
            let output = batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output();
            let (location, size) = if column {
                (Point::new(px(y), px(x)), Size::new(px(height), px(width)))
            } else {
                (Point::new(px(x), px(y)), Size::new(px(width), px(height)))
            };
            assert_eq!(
                (output.location, output.size),
                (location, size),
                "{flow:?}, reordered {reordered}, node {node}"
            );
            if node != 0 {
                assert_eq!(output.source_index.get(), node - 1);
            }
        }
    }
}

fn assert_effective_controls<S: LayoutScalar>() {
    for case in flow_cases() {
        for (_, pack, pack_dense) in pack_cases() {
            let expected = if pack_dense {
                case.dense_pack
            } else {
                case.normal_pack
            };
            let flow = resolve_grid_auto_flow(case.authored, pack).unwrap();
            assert_eq!(flow, expected, "{}, {pack:?}", case.spelling);
            let dense = matches!(expected, GridAutoFlow::RowDense | GridAutoFlow::ColumnDense);
            assert_grid_geometry::<S>(flow, case.column, dense);
        }
    }
}

#[test]
fn effective_controls_reach_ordered_row_and_column_backfill_f32() {
    assert_effective_controls::<f32>();
}

#[test]
fn effective_controls_reach_ordered_row_and_column_backfill_f64() {
    assert_effective_controls::<f64>();
}

fn parsed_controls(text: &str) -> (CssGridAutoFlow, CssItemPack) {
    let report = CssParser::default().parse_block_contents(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
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
                CssContributionValueRef::Ordinary(CssLonghandValueRef::GridAutoFlow(value)) => {
                    auto_flow = *value;
                }
                CssContributionValueRef::Ordinary(CssLonghandValueRef::ItemPack(value)) => {
                    pack = *value;
                }
                _ => {}
            }
        }
    }
    (auto_flow, pack)
}

#[test]
fn parsed_pack_states_keep_independent_density_across_shorthand_and_order() {
    for case in flow_cases() {
        for (pack_text, pack, pack_dense) in pack_cases() {
            for declarations in [
                format!("item-flow:{pack_text};grid-auto-flow:{}", case.spelling),
                format!("grid-auto-flow:{};item-flow:{pack_text}", case.spelling),
                format!("item-pack:{pack_text};grid-auto-flow:{}", case.spelling),
                format!("grid-auto-flow:{};item-pack:{pack_text}", case.spelling),
            ] {
                let controls = parsed_controls(&declarations);
                assert_eq!(controls, (case.authored, pack), "{declarations}");
                let expected = if pack_dense {
                    case.dense_pack
                } else {
                    case.normal_pack
                };
                let flow = resolve_grid_auto_flow(controls.0, controls.1).unwrap();
                assert_eq!(flow, expected, "{declarations}");
                let dense = matches!(expected, GridAutoFlow::RowDense | GridAutoFlow::ColumnDense);
                assert_grid_geometry::<f32>(flow, case.column, dense);
            }
        }
    }
}
