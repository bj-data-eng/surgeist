#[path = "../crates/surgeist-layout/tests/support/mod.rs"]
mod support;

use support::grid_layout_comparison::{GridLayoutComparison, GridLayoutNode};
use support::oracle::{
    grid::{
        AutoPlacer, AxisEdges, ContributionSize, Flow, GridArea, GridAxis, GridTrack,
        ItemContributionFacts, LaneAutoFlow, LaneFlowTolerance, LaneIntrinsicItem,
        LaneIntrinsicSizingInput, LaneItemInput, LanePlacementInput, SubgridItemRectInput,
        TrackMax, TrackMin, TrackSizingSlice, compose_subgrid_item_rect,
    },
    inline,
};
use support::oracle_tree::{OracleMeasurement, OracleTree};
use surgeist::layout::{
    AlignContent, AlignItems, Available, ComputeInput, ComputeOutput, Dimension, Direction,
    Display, Edges, GridAutoFlow, GridAxisKind as ProductionGridAxisKind, GridFlowTolerance,
    GridPlacement, GridTemplateAreaRow, GridTemplateAreas,
    LaneContributionFacts as ProductionLaneContributionFacts,
    LaneIntrinsicItem as ProductionLaneIntrinsicItem,
    LaneIntrinsicSizingInput as ProductionLaneIntrinsicSizingInput, LaneItem as ProductionLaneItem,
    LanePlacementInput as ProductionLanePlacementInput, Length, LengthAuto, MaxTrackSizing,
    MinTrackSizing, NodeInput, Point, Position, RawGridLine, RawGridPlacement, RequestedAxis,
    RunMode, Size, SizingMode, TrackComponent, TrackSizing as ProductionTrackSizing, WritingMode,
    compute_root, lane_intrinsic_sizing as production_lane_intrinsic_sizing,
    place_lanes as production_place_lanes, round_layout,
};

fn fixed_rows(height: f32) -> support::oracle::grid::TrackSizingReport {
    TrackSizingSlice::definite_rows(height, 0.0)
        .track(GridTrack::fixed(height))
        .solve()
}

fn assert_atomic_inline_layout_matches_oracle(display: Display) {
    let item_sizes = [
        ("first", Size::new(20.0, 10.0)),
        ("second", Size::new(30.0, 20.0)),
        ("third", Size::new(15.0, 10.0)),
    ];
    let mut tree = OracleTree::new().children(0, [1, 2, 3]).style(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    for (node, (_, size)) in (1_u32..).zip(item_sizes) {
        tree = tree.style(node, atomic_inline_node_input(display, size));
    }

    compute_root(&mut tree, 0, Size::splat(Available::definite(50.0)));
    round_layout(&mut tree, 0);

    let expected = inline::layout_atomic_inline(inline::AtomicInlineInput {
        available_width: inline::InlineAvailable::Definite(50.0),
        items: item_sizes
            .into_iter()
            .map(|(id, size)| inline::AtomicInlineItemFacts {
                id,
                size: inline::InlineSize::new(size.width, size.height),
                margin: inline::InlineEdges::ZERO,
                first_baseline: None,
            })
            .collect(),
    });

    let root = tree.final_layout(0).expect("root layout");
    assert_layout_close(root.size.width, 50.0, "root width");
    assert_layout_close(root.size.height, expected.size.height, "root height");

    for (index, expected_item) in expected.items.iter().enumerate() {
        let node = (index + 1) as u32;
        let actual = tree.final_layout(node).expect("child layout");
        assert_layout_close(
            actual.location.x,
            expected_item.location.x,
            &format!("node {node} x"),
        );
        assert_layout_close(
            actual.location.y,
            expected_item.location.y,
            &format!("node {node} y"),
        );
        assert_layout_close(
            actual.size.width,
            expected_item.size.width,
            &format!("node {node} width"),
        );
        assert_layout_close(
            actual.size.height,
            expected_item.size.height,
            &format!("node {node} height"),
        );
    }
}

fn atomic_inline_node_input(display: Display, size: Size<f32>) -> NodeInput {
    match display.inner_display() {
        Display::Grid => NodeInput {
            display,
            grid_template_columns: vec![TrackComponent::px(size.width)],
            grid_template_rows: vec![TrackComponent::px(size.height)],
            ..NodeInput::DEFAULT
        },
        Display::GridLanes => NodeInput {
            display,
            grid_template_columns: vec![TrackComponent::px(size.width)],
            grid_template_rows: vec![TrackComponent::px(size.height)],
            ..NodeInput::DEFAULT
        },
        _ => NodeInput {
            display,
            size: Size::new(Dimension::px(size.width), Dimension::px(size.height)),
            ..NodeInput::DEFAULT
        },
    }
}

fn assert_layout_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.000_1,
        "{label}: expected {expected}, got {actual}"
    );
}

fn named_grid_oracle_lines() -> support::oracle::grid::NamedGridLines {
    support::oracle::grid::NamedGridLines::new(
        GridAxis::Column,
        3,
        vec![
            vec!["a", "foo-start"],
            vec!["a", "foo", "foo-end"],
            vec!["a"],
            vec![],
        ],
    )
    .unwrap()
}

fn named_grid_track_components() -> Vec<TrackComponent> {
    vec![
        TrackComponent::line_names(["a", "foo-start"]),
        TrackComponent::px(40.0),
        TrackComponent::line_names(["a", "foo", "foo-end"]),
        TrackComponent::px(40.0),
        TrackComponent::line_names(["a"]),
        TrackComponent::px(40.0),
    ]
}

fn assert_named_grid_column_matches_oracle(
    raw_column: RawGridPlacement,
    oracle_column: support::oracle::grid::NamedAxisPlacement,
    auto_cursor_line: Option<isize>,
    grid_auto_columns: Vec<TrackComponent>,
    label: &str,
) {
    let expected = support::oracle::grid::resolve_named_axis_placement(
        &named_grid_oracle_lines(),
        oracle_column,
        auto_cursor_line,
    )
    .unwrap()
    .resolved;

    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(200.0), Dimension::px(20.0)),
                grid_template_columns: named_grid_track_components(),
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_auto_columns,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: raw_column,
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(200.0), Available::Definite(20.0)),
    );
    round_layout(&mut tree, 1);
    let actual = tree.final_layout(2).expect("child layout");

    assert_layout_close(
        actual.location.x,
        (expected.start_line as f32 - 1.0) * 40.0,
        &format!("{label} x"),
    );
    assert_layout_close(
        actual.size.width,
        expected.span as f32 * 40.0,
        &format!("{label} width"),
    );
}

fn assert_named_grid_column_falls_back_to_auto_when_oracle_rejects(
    raw_column: RawGridPlacement,
    oracle_column: support::oracle::grid::NamedAxisPlacement,
    expected_error: support::oracle::grid::NamedGridError,
) {
    let oracle_error = support::oracle::grid::resolve_named_axis_placement(
        &named_grid_oracle_lines(),
        oracle_column,
        None,
    )
    .unwrap_err();
    assert_eq!(oracle_error, expected_error);

    let layout_for = |raw_grid_column: RawGridPlacement| {
        let mut tree = OracleTree::new()
            .children(1, [2])
            .style(
                1,
                NodeInput {
                    display: Display::Grid,
                    size: Size::new(Dimension::px(200.0), Dimension::px(20.0)),
                    grid_template_columns: named_grid_track_components(),
                    grid_template_rows: vec![TrackComponent::px(20.0)],
                    ..NodeInput::DEFAULT
                },
            )
            .style(
                2,
                NodeInput {
                    raw_grid_column,
                    raw_grid_row: RawGridPlacement::line(1),
                    ..NodeInput::DEFAULT
                },
            );

        compute_root(
            &mut tree,
            1,
            Size::new(Available::Definite(200.0), Available::Definite(20.0)),
        );
        round_layout(&mut tree, 1);
        tree.final_layout(2).expect("child layout")
    };

    let invalid_named = layout_for(raw_column);
    let plain_auto = layout_for(RawGridPlacement::AUTO);

    assert_layout_close(
        invalid_named.location.x,
        plain_auto.location.x,
        "fallback auto x",
    );
    assert_layout_close(
        invalid_named.size.width,
        plain_auto.size.width,
        "fallback auto width",
    );
}

#[test]
fn named_grid_layout_oracle_matches_bare_explicit_and_repeated_names() {
    use support::oracle::grid::{NamedAxisPlacement, NamedGridLine};

    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::BareIdent("foo".to_string()),
            RawGridLine::BareIdent("foo".to_string()),
        ),
        NamedAxisPlacement {
            start: NamedGridLine::BareIdent("foo".to_string()),
            end: NamedGridLine::BareIdent("foo".to_string()),
        },
        None,
        Vec::new(),
        "bare foo",
    );
    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "foo".to_string(),
                index: 1,
            },
            RawGridLine::NamedLine {
                name: "foo".to_string(),
                index: 1,
            },
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: "foo".to_string(),
                occurrence: 1,
            },
            end: NamedGridLine::Named {
                name: "foo".to_string(),
                occurrence: 1,
            },
        },
        None,
        Vec::new(),
        "explicit foo",
    );
    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1,
            },
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: "a".to_string(),
                occurrence: 2,
            },
            end: NamedGridLine::Span {
                name: Some("a".to_string()),
                count: 1,
            },
        },
        None,
        Vec::new(),
        "repeated named span",
    );
}

#[test]
fn named_grid_layout_oracle_matches_negative_missing_and_backward_spans() {
    use support::oracle::grid::{NamedAxisPlacement, NamedGridLine};

    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: -1,
            },
            RawGridLine::Auto,
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: "a".to_string(),
                occurrence: -1,
            },
            end: NamedGridLine::Auto,
        },
        None,
        Vec::new(),
        "negative occurrence",
    );
    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 4,
            },
            RawGridLine::Auto,
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: "a".to_string(),
                occurrence: 4,
            },
            end: NamedGridLine::Auto,
        },
        None,
        vec![TrackComponent::px(40.0)],
        "missing after occurrence",
    );
    assert_named_grid_column_falls_back_to_auto_when_oracle_rejects(
        RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: "missing".to_string(),
                index: -4,
            },
            RawGridLine::Auto,
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Named {
                name: "missing".to_string(),
                occurrence: -4,
            },
            end: NamedGridLine::Auto,
        },
        support::oracle::grid::NamedGridError::LineBeforeFirst {
            axis: GridAxis::Column,
            start_line: -3,
            end_line: -2,
        },
    );
    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Line(3),
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Span {
                name: Some("a".to_string()),
                count: 2,
            },
            end: NamedGridLine::Number(3),
        },
        None,
        Vec::new(),
        "backward named span",
    );
}

#[test]
fn named_grid_layout_oracle_matches_auto_span_and_conflict_normalization() {
    use support::oracle::grid::{NamedAxisPlacement, NamedGridLine};

    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Auto,
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Span {
                name: Some("a".to_string()),
                count: 2,
            },
            end: NamedGridLine::Auto,
        },
        Some(1),
        Vec::new(),
        "lone named span",
    );
    assert_named_grid_column_matches_oracle(
        RawGridPlacement::new(
            RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2,
            },
            RawGridLine::Span(3),
        ),
        NamedAxisPlacement {
            start: NamedGridLine::Span {
                name: Some("a".to_string()),
                count: 2,
            },
            end: NamedGridLine::Span {
                name: None,
                count: 3,
            },
        },
        Some(1),
        Vec::new(),
        "mixed spans",
    );
    assert_named_grid_column_matches_oracle(
        RawGridPlacement::lines(3, 1),
        NamedAxisPlacement {
            start: NamedGridLine::Number(3),
            end: NamedGridLine::Number(1),
        },
        None,
        Vec::new(),
        "start after end",
    );
    assert_named_grid_column_matches_oracle(
        RawGridPlacement::lines(2, 2),
        NamedAxisPlacement {
            start: NamedGridLine::Number(2),
            end: NamedGridLine::Number(2),
        },
        None,
        Vec::new(),
        "equal lines",
    );
}

#[test]
fn named_grid_layout_oracle_matches_template_area_generated_lines() {
    use support::oracle::grid::{NamedAxisPlacement, NamedGridLine};

    let areas = support::oracle::grid::TemplateAreas::new([vec!["foo", "foo", "bar"]]).unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(GridAxis::Column, 3),
    )
    .unwrap();
    let expected = support::oracle::grid::resolve_named_axis_placement(
        &columns,
        NamedAxisPlacement {
            start: NamedGridLine::BareIdent("foo".to_string()),
            end: NamedGridLine::BareIdent("foo".to_string()),
        },
        None,
    )
    .unwrap()
    .resolved;

    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            Some("foo".to_string()),
                            Some("foo".to_string()),
                            Some("bar".to_string()),
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("foo".to_string()),
                    RawGridLine::BareIdent("foo".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(120.0), Available::Definite(20.0)),
    );
    round_layout(&mut tree, 1);
    let child = tree.final_layout(2).expect("child layout");

    assert_layout_close(
        child.location.x,
        (expected.start_line as f32 - 1.0) * 40.0,
        "area generated x",
    );
    assert_layout_close(
        child.size.width,
        expected.span as f32 * 40.0,
        "area generated width",
    );
}

#[test]
fn subgrid_layout_oracle_matches_merged_local_and_inherited_area_lines() {
    use support::oracle::grid::{NamedAxisPlacement, NamedGridLine, TrackSpan};

    let parent_areas =
        support::oracle::grid::TemplateAreas::new([vec![".", "parent", "parent", "."]]).unwrap();
    let parent_facts = support::oracle::grid::area_generated_facts(
        &parent_areas,
        support::oracle::grid::NamedGridLines::empty(GridAxis::Column, 4),
        support::oracle::grid::NamedGridLines::empty(GridAxis::Row, 2),
    )
    .unwrap();
    let inherited = support::oracle::grid::inherit_named_subgrid_lines(
        &parent_facts.columns,
        TrackSpan::new(2, 4),
        false,
        vec![vec![], vec![], vec![]],
        Some(&parent_facts),
    )
    .unwrap();
    let local_areas = support::oracle::grid::TemplateAreas::new([vec!["local", "local"]]).unwrap();
    let merged_columns = support::oracle::grid::area_generated_lines(
        GridAxis::Column,
        &local_areas,
        inherited.lines,
    )
    .unwrap();

    let expected_local = support::oracle::grid::resolve_named_axis_placement(
        &merged_columns,
        NamedAxisPlacement {
            start: NamedGridLine::BareIdent("local".to_string()),
            end: NamedGridLine::BareIdent("local".to_string()),
        },
        None,
    )
    .unwrap()
    .resolved;
    let expected_parent = support::oracle::grid::resolve_named_axis_placement(
        &merged_columns,
        NamedAxisPlacement {
            start: NamedGridLine::BareIdent("parent".to_string()),
            end: NamedGridLine::BareIdent("parent".to_string()),
        },
        None,
    )
    .unwrap()
    .resolved;

    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3, 4])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(160.0), Dimension::px(40.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            None,
                            Some("parent".to_string()),
                            Some("parent".to_string()),
                            None,
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::lines(2, 4),
                grid_row: GridPlacement::lines(1, 3),
                grid_template_columns: vec![TrackComponent::Subgrid(
                    surgeist::layout::SubgridTrack {
                        name_components: Vec::new(),
                    },
                )],
                grid_template_rows: vec![TrackComponent::px(20.0), TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![Some("local".to_string()), Some("local".to_string())],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("local".to_string()),
                    RawGridLine::BareIdent("local".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("parent".to_string()),
                    RawGridLine::BareIdent("parent".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(2),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(160.0), Available::Definite(40.0)),
    );
    round_layout(&mut tree, 1);

    for (node, expected, label) in [
        (3, expected_local, "local area"),
        (4, expected_parent, "inherited area"),
    ] {
        let child = tree.final_layout(node).expect("child layout");
        assert_layout_close(
            child.location.x,
            (expected.start_line as f32 - 1.0) * 40.0,
            &format!("{label} x"),
        );
        assert_layout_close(
            child.size.width,
            expected.span as f32 * 40.0,
            &format!("{label} width"),
        );
    }
}

#[test]
fn subgrid_layout_oracle_matches_local_area_clamp_to_inherited_span() {
    use support::oracle::grid::{NamedAxisPlacement, NamedGridLine};

    let clamped_local_areas =
        support::oracle::grid::TemplateAreas::new([vec!["wide", "wide"]]).unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        GridAxis::Column,
        &clamped_local_areas,
        support::oracle::grid::NamedGridLines::empty(GridAxis::Column, 2),
    )
    .unwrap();
    let expected = support::oracle::grid::resolve_named_axis_placement(
        &columns,
        NamedAxisPlacement {
            start: NamedGridLine::BareIdent("wide".to_string()),
            end: NamedGridLine::BareIdent("wide".to_string()),
        },
        None,
    )
    .unwrap()
    .resolved;

    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(160.0), Dimension::px(20.0)),
                grid_template_columns: vec![
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                    TrackComponent::px(40.0),
                ],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::lines(1, 3),
                grid_row: GridPlacement::line(1),
                grid_template_columns: vec![TrackComponent::Subgrid(
                    surgeist::layout::SubgridTrack {
                        name_components: Vec::new(),
                    },
                )],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                grid_template_areas: GridTemplateAreas {
                    rows: vec![GridTemplateAreaRow {
                        cells: vec![
                            Some("wide".to_string()),
                            Some("wide".to_string()),
                            Some("wide".to_string()),
                            Some("wide".to_string()),
                        ],
                    }],
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(
                    RawGridLine::BareIdent("wide".to_string()),
                    RawGridLine::BareIdent("wide".to_string()),
                ),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(160.0), Available::Definite(20.0)),
    );
    round_layout(&mut tree, 1);
    let child = tree.final_layout(3).expect("child layout");

    assert_layout_close(
        child.location.x,
        (expected.start_line as f32 - 1.0) * 40.0,
        "clamped local area x",
    );
    assert_layout_close(
        child.size.width,
        expected.span as f32 * 40.0,
        "clamped local area width",
    );
}

#[test]
fn oracle_layout_inline_block_line_matches_layout() {
    assert_atomic_inline_layout_matches_oracle(Display::InlineBlock);
}

#[test]
fn oracle_layout_inline_grid_line_matches_layout() {
    assert_atomic_inline_layout_matches_oracle(Display::InlineGrid);
}

#[test]
fn oracle_layout_inline_grid_lanes_line_matches_layout() {
    assert_atomic_inline_layout_matches_oracle(Display::InlineGridLanes);
}

#[test]
fn oracle_layout_fixed_tracks_match_layout_child_rects() {
    let expected_columns = TrackSizingSlice::definite_columns(210.0, 10.0)
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(120.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(40.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(210.0, 40.0))
        .columns(vec![TrackComponent::px(80.0), TrackComponent::px(120.0)])
        .rows(vec![TrackComponent::px(40.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_percent_and_flex_tracks_match_layout_child_rects() {
    let expected_columns = TrackSizingSlice::definite_columns(400.0, 20.0)
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::percent(0.25))
        .track(GridTrack::flex(1.0))
        .track(GridTrack::flex(3.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(400.0, 40.0))
        .columns(vec![
            TrackComponent::px(80.0),
            TrackComponent::percent(0.25),
            TrackComponent::fr(1.0),
            TrackComponent::fr(3.0),
        ])
        .rows(vec![TrackComponent::px(40.0)])
        .gap(Size::new(20.0, 0.0))
        .expected_tracks(expected_columns, fixed_rows(40.0))
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .child(GridArea::new(3, 1, 1, 1))
        .child(GridArea::new(4, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_sub_one_flex_track_uses_partial_leftover_space() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 0.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::flex(0.5))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![TrackComponent::px(50.0), TrackComponent::fr(0.5)])
        .rows(vec![TrackComponent::px(30.0)])
        .expected_tracks(expected_columns, fixed_rows(30.0))
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_minmax_tracks_match_layout_child_rects() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 0.0)
        .track(GridTrack::new(
            support::oracle::grid::TrackMin::Fixed(40.0),
            support::oracle::grid::TrackMax::Fixed(90.0),
        ))
        .track(GridTrack::new(
            support::oracle::grid::TrackMin::Percent(0.25),
            support::oracle::grid::TrackMax::Auto,
        ))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::minmax(MinTrackSizing::px(40.0), MaxTrackSizing::px(90.0)),
            TrackComponent::minmax(MinTrackSizing::percent(0.25), MaxTrackSizing::AUTO),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .expected_tracks(expected_columns, fixed_rows(30.0))
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_stretch_expands_auto_tracks_like_layout() {
    let expected_columns = TrackSizingSlice::definite_columns(120.0, 20.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .stretch_auto_tracks()
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(120.0, 30.0))
        .columns(vec![TrackComponent::AUTO, TrackComponent::AUTO])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(20.0, 0.0))
        .justify_content(AlignContent::Stretch)
        .expected_tracks(expected_columns, fixed_rows(30.0))
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_explicit_line_span_matches_layout_area_rect() {
    let expected_columns = TrackSizingSlice::definite_columns(250.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::fixed(70.0))
        .track(GridTrack::fixed(110.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(250.0, 40.0))
        .columns(vec![
            TrackComponent::px(50.0),
            TrackComponent::px(70.0),
            TrackComponent::px(110.0),
        ])
        .rows(vec![TrackComponent::px(40.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, fixed_rows(40.0))
        .child(GridArea::new(2, 1, 2, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_row_auto_flow_matches_oracle_placement() {
    let mut placement = AutoPlacer::try_new(2, 2, Flow::Row).unwrap();
    let first = placement.place(1, 1).unwrap();
    let second = placement.place(1, 1).unwrap();
    let third = placement.place(1, 1).unwrap();
    let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::fixed(50.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(45.0, 5.0)
        .track(GridTrack::fixed(20.0))
        .track(GridTrack::fixed(20.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(110.0, 45.0))
        .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
        .rows(vec![TrackComponent::px(20.0), TrackComponent::px(20.0)])
        .gap(Size::new(10.0, 5.0))
        .expected_tracks(expected_columns, expected_rows)
        .auto_child(first)
        .auto_child(second)
        .auto_child(third)
        .assert_layout();
}

#[test]
fn oracle_layout_column_auto_flow_matches_oracle_placement() {
    let mut placement = AutoPlacer::try_new(2, 2, Flow::Column).unwrap();
    let first = placement.place(1, 1).unwrap();
    let second = placement.place(1, 1).unwrap();
    let third = placement.place(1, 1).unwrap();
    let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::fixed(50.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(45.0, 5.0)
        .track(GridTrack::fixed(20.0))
        .track(GridTrack::fixed(20.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(110.0, 45.0))
        .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
        .rows(vec![TrackComponent::px(20.0), TrackComponent::px(20.0)])
        .gap(Size::new(10.0, 5.0))
        .auto_flow(GridAutoFlow::Column)
        .expected_tracks(expected_columns, expected_rows)
        .auto_child(first)
        .auto_child(second)
        .auto_child(third)
        .assert_layout();
}

#[test]
fn oracle_layout_dense_auto_flow_matches_spanning_oracle_placement() {
    let mut placement = AutoPlacer::try_new(3, 2, Flow::RowDense).unwrap();
    let first = placement.place(2, 1).unwrap();
    let second = placement.place(2, 1).unwrap();
    let third = placement.place(1, 1).unwrap();
    let expected_columns = TrackSizingSlice::definite_columns(150.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(40.0, 0.0)
        .track(GridTrack::fixed(20.0))
        .track(GridTrack::fixed(20.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(150.0, 40.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(50.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(20.0), TrackComponent::px(20.0)])
        .auto_flow(GridAutoFlow::RowDense)
        .expected_tracks(expected_columns, expected_rows)
        .auto_spanning_child(first, 2, 1)
        .auto_spanning_child(second, 2, 1)
        .auto_child(third)
        .assert_layout();
}

#[test]
fn oracle_layout_center_alignment_offsets_tracks_like_layout() {
    let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::fixed(50.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .justify_content(AlignContent::Center)
        .expected_tracks(expected_columns, fixed_rows(30.0))
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_space_between_alignment_offsets_tracks_like_layout() {
    let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::fixed(50.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .justify_content(AlignContent::SpaceBetween)
        .expected_tracks(expected_columns, fixed_rows(30.0))
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_safe_center_alignment_falls_back_on_overflow() {
    let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::fixed(50.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(80.0, 30.0))
        .columns(vec![TrackComponent::px(50.0), TrackComponent::px(50.0)])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .justify_content(AlignContent::SafeCenter)
        .expected_tracks(expected_columns, fixed_rows(30.0))
        .child(GridArea::new(1, 1, 1, 1))
        .child(GridArea::new(2, 1, 1, 1))
        .assert_layout();
}

#[test]
fn oracle_layout_auto_track_uses_supplied_intrinsic_measurement() {
    let expected_columns = TrackSizingSlice::definite_columns(80.0, 0.0)
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 80.0,
            max_content: 80.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(80.0, 20.0))
        .columns(vec![TrackComponent::AUTO])
        .rows(vec![TrackComponent::px(20.0)])
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .measured_child(GridArea::new(1, 1, 1, 1), Size::new(80.0, 10.0))
        .assert_layout();
}

#[test]
fn oracle_layout_spanning_auto_tracks_distribute_intrinsic_deficit() {
    let expected_columns = TrackSizingSlice::definite_columns(110.0, 10.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 2, 1),
            min_content: 110.0,
            max_content: 110.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(110.0, 20.0))
        .columns(vec![TrackComponent::AUTO, TrackComponent::AUTO])
        .rows(vec![TrackComponent::px(20.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .measured_child(GridArea::new(1, 1, 2, 1), Size::new(110.0, 10.0))
        .assert_layout();
}

#[test]
fn oracle_layout_fit_content_track_clamps_intrinsic_growth() {
    let expected_columns = TrackSizingSlice::definite_columns(40.0, 0.0)
        .track(GridTrack::new(TrackMin::Auto, TrackMax::FitContent(40.0)))
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 90.0,
            max_content: 90.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(40.0, 20.0))
        .columns(vec![TrackComponent::minmax(
            MinTrackSizing::AUTO,
            MaxTrackSizing::fit_content(Length::px(40.0)),
        )])
        .rows(vec![TrackComponent::px(20.0)])
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .measured_child(GridArea::new(1, 1, 1, 1), Size::new(90.0, 10.0))
        .assert_layout();
}

#[test]
fn oracle_layout_harness_asserts_nested_grid_descendant_output() {
    let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
        .track(GridTrack::fixed(120.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(60.0, 0.0)
        .track(GridTrack::fixed(60.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(120.0, 60.0))
        .columns(vec![TrackComponent::px(120.0)])
        .rows(vec![TrackComponent::px(60.0)])
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::grid(GridArea::new(1, 1, 1, 1))
                .margin(Edges::new(
                    LengthAuto::px(6.0),
                    LengthAuto::px(4.0),
                    LengthAuto::px(2.0),
                    LengthAuto::px(10.0),
                ))
                .expect_layout(Point::new(10.0, 6.0), Size::new(106.0, 52.0))
                .columns(vec![TrackComponent::px(30.0), TrackComponent::px(76.0)])
                .rows(vec![TrackComponent::px(52.0)])
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .size(Size::new(Dimension::px(76.0), Dimension::px(52.0)))
                        .expect_layout(Point::new(30.0, 0.0), Size::new(76.0, 52.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_child_rect_matches_oracle_composed_rect() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();
    let rect = compose_subgrid_item_rect(SubgridItemRectInput {
        inherited_axis: GridAxis::Column,
        inherited_axis_offset: 50.0,
        standalone_axis_offset: 0.0,
        inherited_axis_size: 60.0,
        standalone_axis_size: 30.0,
        container_mbp_offset: AxisEdges {
            start: 0.0,
            end: 0.0,
        },
        item_inline_offset: 90.0,
        item_block_offset: 0.0,
    })
    .rect;

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .expect_layout(
                            Point::new(rect.x - 50.0, rect.y),
                            Size::new(rect.width, rect.height),
                        )
                        .expect_final_layout(
                            Point::new(rect.x - 50.0, rect.y),
                            Size::new(rect.width, rect.height),
                        ),
                ),
        )
        .assert_layout();
}

#[ignore = "enable after production baseline helper exists"]
#[test]
fn layout_oracle_grid_baseline_offset_matches_oracle() {
    let oracle_offset = support::oracle::grid::baseline_offset(
        support::oracle::grid::BaselineGroupKind::Major,
        20.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 11.0,
        },
    );

    // Compare the future production baseline offset helper to `oracle::grid::baseline_offset`.
    assert_eq!(oracle_offset, 9.0);
}

#[test]
fn subgrid_child_items_resolve_against_local_lines() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .expect_layout(Point::new(0.0, 0.0), Size::new(80.0, 30.0))
                        .expect_final_layout(Point::new(0.0, 0.0), Size::new(80.0, 30.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_standalone_axis_uses_ordinary_child_tracks() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(50.0, 0.0)
        .track(GridTrack::fixed(50.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 50.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(50.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .rows(vec![TrackComponent::px(12.0), TrackComponent::px(18.0)])
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 50.0))
                .child(
                    GridLayoutNode::item(GridArea::new(1, 2, 1, 1))
                        .expect_layout(Point::new(0.0, 12.0), Size::new(80.0, 18.0))
                        .expect_final_layout(Point::new(0.0, 12.0), Size::new(80.0, 18.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_item_still_respects_parent_grid_placement() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0)),
        )
        .assert_layout();
}

#[test]
fn subgrid_child_auto_margins_use_inherited_area_size() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .size(Size::new(Dimension::px(20.0), Dimension::px(30.0)))
                        .margin(Edges::new(
                            LengthAuto::px(0.0),
                            LengthAuto::auto(),
                            LengthAuto::px(0.0),
                            LengthAuto::auto(),
                        ))
                        .expect_layout(Point::new(110.0, 0.0), Size::new(20.0, 30.0))
                        .expect_final_layout(Point::new(110.0, 0.0), Size::new(20.0, 30.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_child_alignment_uses_inherited_area_size() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .size(Size::new(Dimension::px(20.0), Dimension::px(10.0)))
                        .justify_self(AlignItems::Center)
                        .align_self(AlignItems::End)
                        .expect_layout(Point::new(110.0, 20.0), Size::new(20.0, 10.0))
                        .expect_final_layout(Point::new(110.0, 20.0), Size::new(20.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_rtl_child_lines_use_reversed_inherited_columns() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .direction(Direction::Rtl)
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .expect_layout(Point::new(90.0, 0.0), Size::new(60.0, 30.0))
                        .expect_final_layout(Point::new(90.0, 0.0), Size::new(60.0, 30.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_explicit_zero_gap_overrides_parent_gap() {
    let expected_columns = TrackSizingSlice::definite_columns(220.0, 20.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(220.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(20.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .gap(Size::new(Length::ZERO, Length::ZERO))
                .expect_layout(Point::new(60.0, 0.0), Size::new(160.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .expect_layout(Point::new(90.0, 0.0), Size::new(70.0, 30.0))
                        .expect_final_layout(Point::new(90.0, 0.0), Size::new(70.0, 30.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_percent_gap_uses_content_box_basis() {
    let expected_columns = TrackSizingSlice::definite_columns(220.0, 20.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(220.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(20.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .padding(Edges::new(
                    Length::ZERO,
                    Length::percent(0.1),
                    Length::ZERO,
                    Length::percent(0.1),
                ))
                .gap(Size::new(Length::percent(0.1), Length::ZERO))
                .expect_layout(Point::new(60.0, 0.0), Size::new(160.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .expect_layout(Point::new(96.4, 0.0), Size::new(47.6, 30.0))
                        .expect_final_layout(Point::new(96.0, 0.0), Size::new(48.0, 30.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_percentage_padding_uses_grid_area_basis() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .padding(Edges::new(
                    Length::ZERO,
                    Length::percent(0.1),
                    Length::ZERO,
                    Length::percent(0.1),
                ))
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .expect_layout(Point::new(15.0, 0.0), Size::new(65.0, 30.0))
                        .expect_final_layout(Point::new(15.0, 0.0), Size::new(65.0, 30.0)),
                ),
        )
        .assert_layout();
}

fn intrinsic_item(area: GridArea, size: f32) -> ItemContributionFacts {
    ItemContributionFacts {
        area,
        min_content: size,
        max_content: size,
        preferred: ContributionSize::Auto,
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Auto,
        margin_before: 0.0,
        margin_after: 0.0,
        automatic_minimum_applies: true,
    }
}

#[test]
fn subgrid_traversal_nested_inherited_leaf_contribution_grows_parent_auto_track() {
    let expected_columns = TrackSizingSlice::definite_columns(90.0, 0.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 90.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(90.0, 20.0))
        .columns(vec![
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
        ])
        .rows(vec![TrackComponent::px(20.0)])
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                .expect_layout(Point::new(0.0, 0.0), Size::new(90.0, 20.0))
                .child(
                    GridLayoutNode::subgrid(GridArea::new(2, 1, 1, 1))
                        .expect_layout(Point::new(0.0, 0.0), Size::new(90.0, 20.0))
                        .child(
                            GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                                .measurement(Size::new(90.0, 10.0))
                                .expect_layout(Point::new(0.0, 0.0), Size::new(90.0, 10.0)),
                        ),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_reversed_nested_inherited_subgrid_maps_to_mirrored_track() {
    let expected_columns = TrackSizingSlice::definite_columns(80.0, 0.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 80.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(80.0, 20.0))
        .columns(vec![
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
        ])
        .rows(vec![TrackComponent::px(20.0)])
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                .direction(Direction::Rtl)
                .expect_layout(Point::new(0.0, 0.0), Size::new(80.0, 20.0))
                .child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .measurement(Size::new(80.0, 10.0))
                        .expect_layout(Point::new(0.0, 0.0), Size::new(80.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_nested_margin_border_padding_increases_contribution() {
    let expected_columns = TrackSizingSlice::definite_columns(85.0, 0.0)
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(1, 1, 1, 1), 85.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(85.0, 20.0))
        .columns(vec![TrackComponent::AUTO])
        .rows(vec![TrackComponent::px(20.0)])
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 1, 1))
                .margin(Edges::new(
                    LengthAuto::px(0.0),
                    LengthAuto::px(8.0),
                    LengthAuto::px(0.0),
                    LengthAuto::px(5.0),
                ))
                .border(Edges::new(
                    Length::ZERO,
                    Length::px(9.0),
                    Length::ZERO,
                    Length::px(6.0),
                ))
                .padding(Edges::new(
                    Length::ZERO,
                    Length::px(10.0),
                    Length::ZERO,
                    Length::px(7.0),
                ))
                .expect_layout(Point::new(5.0, 0.0), Size::new(72.0, 20.0))
                .child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .measurement(Size::new(40.0, 10.0))
                        .expect_layout(Point::new(13.0, 0.0), Size::new(40.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_gap_difference_adjustment_accumulates_through_nested_subgrids() {
    let expected_columns = TrackSizingSlice::definite_columns(70.0, 10.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 50.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(70.0, 20.0))
        .columns(vec![
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
        ])
        .rows(vec![TrackComponent::px(20.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                .gap(Size::new(Length::px(20.0), Length::ZERO))
                .expect_layout(Point::new(0.0, 0.0), Size::new(70.0, 20.0))
                .child(
                    GridLayoutNode::subgrid(GridArea::new(2, 1, 1, 1))
                        .gap(Size::new(Length::px(28.0), Length::ZERO))
                        .expect_layout(Point::new(15.0, 0.0), Size::new(60.0, 20.0))
                        .child(
                            GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                                .measurement(Size::new(40.0, 10.0))
                                .expect_layout(Point::new(0.0, 0.0), Size::new(40.0, 10.0)),
                        ),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_direct_leaf_uses_internal_gap_adjustment() {
    let expected_columns = TrackSizingSlice::definite_columns(70.0, 10.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 50.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(70.0, 20.0))
        .columns(vec![
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
        ])
        .rows(vec![TrackComponent::px(20.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                .gap(Size::new(Length::px(20.0), Length::ZERO))
                .expect_layout(Point::new(0.0, 0.0), Size::new(70.0, 20.0))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .measurement(Size::new(40.0, 10.0))
                        .expect_layout(Point::new(15.0, 0.0), Size::new(40.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_unsupported_sibling_does_not_drop_valid_contribution() {
    let expected_columns = TrackSizingSlice::definite_columns(140.0, 10.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(1, 1, 1, 1), 30.0))
        .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 90.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(140.0, 20.0))
        .columns(vec![
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
        ])
        .rows(vec![TrackComponent::px(20.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 1, 1)).child(
                GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                    .measurement(Size::new(30.0, 10.0))
                    .expect_layout(Point::new(0.0, 0.0), Size::new(30.0, 10.0)),
            ),
        )
        .node(
            GridLayoutNode::subgrid(GridArea::new(3, 1, 1, 1))
                .writing_mode(WritingMode::VerticalRl)
                .child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .measurement(Size::new(90.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_percent_padding_uses_definite_area_basis() {
    let expected_columns = TrackSizingSlice::definite_columns(100.0, 0.0)
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(1, 1, 1, 1), 60.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(100.0, 20.0))
        .columns(vec![TrackComponent::AUTO])
        .rows(vec![TrackComponent::px(20.0)])
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 1, 1))
                .padding(Edges::new(
                    Length::ZERO,
                    Length::percent(0.1),
                    Length::ZERO,
                    Length::percent(0.1),
                ))
                .child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .measurement(Size::new(40.0, 10.0))
                        .expect_layout(Point::new(6.0, 0.0), Size::new(40.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_percent_gap_uses_definite_content_box_basis() {
    let expected_columns = TrackSizingSlice::definite_columns(100.0, 10.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(2, 1, 1, 1), 50.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(100.0, 20.0))
        .columns(vec![
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
        ])
        .rows(vec![TrackComponent::px(20.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(1, 1, 3, 1))
                .gap(Size::new(Length::percent(0.2), Length::ZERO))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .measurement(Size::new(40.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_traversal_translated_nested_edge_adjustments_land_on_ancestor_tracks() {
    let expected_columns = TrackSizingSlice::definite_columns(48.0, 0.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(intrinsic_item(GridArea::new(3, 1, 1, 1), 48.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(48.0, 20.0))
        .columns(vec![
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
            TrackComponent::AUTO,
        ])
        .rows(vec![TrackComponent::px(20.0)])
        .expected_tracks(expected_columns, fixed_rows(20.0))
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 3, 1))
                .expect_layout(Point::new(0.0, 0.0), Size::new(48.0, 20.0))
                .child(
                    GridLayoutNode::subgrid(GridArea::new(2, 1, 1, 1))
                        .margin(Edges::new(
                            LengthAuto::px(0.0),
                            LengthAuto::px(5.0),
                            LengthAuto::px(0.0),
                            LengthAuto::px(3.0),
                        ))
                        .expect_layout(Point::new(3.0, 0.0), Size::new(40.0, 20.0))
                        .child(
                            GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                                .measurement(Size::new(40.0, 10.0))
                                .expect_layout(Point::new(0.0, 0.0), Size::new(40.0, 10.0)),
                        ),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_absolute_descendant_uses_existing_static_position_behavior() {
    let expected_columns = TrackSizingSlice::definite_columns(200.0, 10.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::fixed(60.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .container(Size::new(200.0, 30.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(80.0),
            TrackComponent::px(60.0),
        ])
        .rows(vec![TrackComponent::px(30.0)])
        .gap(Size::new(10.0, 0.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::subgrid(GridArea::new(2, 1, 2, 1))
                .expect_layout(Point::new(50.0, 0.0), Size::new(150.0, 30.0))
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .position(Position::Absolute)
                        .size(Size::new(Dimension::px(10.0), Dimension::px(10.0)))
                        .expect_layout(Point::new(90.0, 0.0), Size::new(10.0, 10.0))
                        .expect_final_layout(Point::new(90.0, 0.0), Size::new(10.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn subgrid_named_placement_clamp_matches_oracle() {
    let oracle = support::oracle::grid::resolve_named_subgrid_axis_placement(
        &support::oracle::grid::NamedGridLines::empty(GridAxis::Column, 1),
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(2),
            end: support::oracle::grid::NamedGridLine::Span {
                name: None,
                count: 3,
            },
        },
        None,
    )
    .unwrap();

    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(40.0), Dimension::px(20.0)),
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::lines(1, 2),
                grid_row: GridPlacement::line(1),
                grid_template_columns: vec![TrackComponent::Subgrid(
                    surgeist::layout::SubgridTrack {
                        name_components: Vec::new(),
                    },
                )],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                raw_grid_column: RawGridPlacement::new(RawGridLine::Line(2), RawGridLine::Span(3)),
                raw_grid_row: RawGridPlacement::line(1),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        1,
        Size::new(Available::Definite(40.0), Available::Definite(20.0)),
    );
    round_layout(&mut tree, 1);

    let child = tree
        .final_layout(3)
        .expect("subgrid child should be laid out");
    assert_eq!(oracle.clamped.resolved.start_line, 1);
    assert_eq!(oracle.clamped.resolved.end_line, 2);
    assert_eq!(child.location.x, 0.0);
    assert_eq!(
        child.size.width,
        (oracle.clamped.resolved.end_line - oracle.clamped.resolved.start_line) as f32 * 40.0
    );
}

#[test]
fn oracle_layout_harness_can_compare_lane_reports() {
    let placement_report = support::oracle::grid::place_lanes(LanePlacementInput {
        grid_axis_tracks: 2,
        auto_flow: LaneAutoFlow::Row,
        lane_gap: 4.0,
        tolerance: LaneFlowTolerance::Fixed(0.0),
        tolerance_basis: 100.0,
        items: vec![
            LaneItemInput::definite("a", 1, 1, 20.0),
            LaneItemInput::auto("b", 1, 10.0),
        ],
    })
    .unwrap();
    let intrinsic_report = support::oracle::grid::lane_intrinsic_sizing(LaneIntrinsicSizingInput {
        axis: GridAxis::Column,
        available: Some(120.0),
        gap: 0.0,
        tracks: vec![GridTrack::auto(), GridTrack::fixed(40.0)],
        content_sized_tracks: vec![0],
        items: vec![LaneIntrinsicItem::indefinite(
            "b",
            1,
            ItemContributionFacts {
                area: GridArea::new(1, 1, 1, 1),
                min_content: 24.0,
                max_content: 30.0,
                preferred: ContributionSize::Auto,
                min_size: ContributionSize::Definite(18.0),
                max_size: ContributionSize::Infinite,
                margin_before: 0.0,
                margin_after: 0.0,
                automatic_minimum_applies: false,
            },
        )],
    })
    .unwrap();

    GridLayoutComparison::new()
        .expect_lane_placement_report(placement_report.clone())
        .expect_lane_intrinsic_report(intrinsic_report.clone())
        .assert_lane_reports(&[placement_report], &[intrinsic_report]);
}

#[test]
fn lanes_row_auto_flow_matches_oracle_placement() {
    assert_production_lane_placement_matches_oracle(
        ProductionLanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: GridAutoFlow::Row,
            lane_gap: 10.0,
            tolerance: GridFlowTolerance::Length(Length::px(0.0)),
            tolerance_basis: 0.0,
            items: vec![
                production_auto_lane_item("a", 1, 20.0),
                production_auto_lane_item("b", 1, 30.0),
                production_auto_lane_item("c", 2, 10.0),
            ],
        },
        LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: LaneAutoFlow::Row,
            lane_gap: 10.0,
            tolerance: LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                LaneItemInput::auto("a", 1, 20.0),
                LaneItemInput::auto("b", 1, 30.0),
                LaneItemInput::auto("c", 2, 10.0),
            ],
        },
    );
}

#[test]
fn lanes_column_auto_flow_matches_oracle_placement() {
    assert_production_lane_placement_matches_oracle(
        ProductionLanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: GridAutoFlow::Column,
            lane_gap: 4.0,
            tolerance: GridFlowTolerance::Length(Length::px(0.0)),
            tolerance_basis: 0.0,
            items: vec![
                production_auto_lane_item("a", 1, 10.0),
                production_auto_lane_item("b", 1, 20.0),
                production_auto_lane_item("c", 1, 30.0),
            ],
        },
        LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: LaneAutoFlow::Column,
            lane_gap: 4.0,
            tolerance: LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                LaneItemInput::auto("a", 1, 10.0),
                LaneItemInput::auto("b", 1, 20.0),
                LaneItemInput::auto("c", 1, 30.0),
            ],
        },
    );
}

#[test]
fn lanes_definite_grid_axis_item_matches_oracle_placement() {
    assert_production_lane_placement_matches_oracle(
        ProductionLanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: GridAutoFlow::Row,
            lane_gap: 5.0,
            tolerance: GridFlowTolerance::Length(Length::px(0.0)),
            tolerance_basis: 0.0,
            items: vec![
                production_definite_lane_item("a", 2, 2, 40.0),
                production_auto_lane_item("b", 1, 20.0),
            ],
        },
        LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: LaneAutoFlow::Row,
            lane_gap: 5.0,
            tolerance: LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                LaneItemInput::definite("a", 2, 2, 40.0),
                LaneItemInput::auto("b", 1, 20.0),
            ],
        },
    );
}

#[test]
fn lanes_auto_span_clamping_matches_oracle_placement() {
    assert_production_lane_placement_matches_oracle(
        ProductionLanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: GridAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: GridFlowTolerance::Length(Length::px(0.0)),
            tolerance_basis: 0.0,
            items: vec![production_auto_lane_item("a", 7, 10.0)],
        },
        LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![LaneItemInput::auto("a", 7, 10.0)],
        },
    );
}

#[test]
fn lanes_finite_tolerance_matches_oracle_placement() {
    assert_production_lane_placement_matches_oracle(
        ProductionLanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: GridAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: GridFlowTolerance::Length(Length::px(10.0)),
            tolerance_basis: 0.0,
            items: vec![
                production_definite_lane_item("a", 1, 1, 10.0),
                production_definite_lane_item("b", 2, 1, 20.0),
                production_auto_lane_item("c", 1, 10.0),
            ],
        },
        LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: LaneFlowTolerance::Fixed(10.0),
            tolerance_basis: 0.0,
            items: vec![
                LaneItemInput::definite("a", 1, 1, 10.0),
                LaneItemInput::definite("b", 2, 1, 20.0),
                LaneItemInput::auto("c", 1, 10.0),
            ],
        },
    );
}

#[test]
fn lanes_finite_search_does_not_wrap_candidate_span_across_grid_axis_end() {
    assert_production_lane_placement_matches_oracle(
        ProductionLanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: GridAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: GridFlowTolerance::Length(Length::px(0.0)),
            tolerance_basis: 0.0,
            items: vec![
                production_auto_lane_item("a", 2, 10.0),
                production_auto_lane_item("b", 2, 10.0),
            ],
        },
        LanePlacementInput {
            grid_axis_tracks: 3,
            auto_flow: LaneAutoFlow::Row,
            lane_gap: 0.0,
            tolerance: LaneFlowTolerance::Fixed(0.0),
            tolerance_basis: 0.0,
            items: vec![
                LaneItemInput::auto("a", 2, 10.0),
                LaneItemInput::auto("b", 2, 10.0),
            ],
        },
    );
}

#[test]
fn lanes_infinite_tolerance_matches_oracle_placement() {
    assert_production_lane_placement_matches_oracle(
        ProductionLanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: GridAutoFlow::Column,
            lane_gap: 0.0,
            tolerance: GridFlowTolerance::Infinite,
            tolerance_basis: 0.0,
            items: vec![
                production_auto_lane_item("a", 1, 10.0),
                production_auto_lane_item("b", 1, 10.0),
                production_auto_lane_item("c", 1, 10.0),
            ],
        },
        LanePlacementInput {
            grid_axis_tracks: 2,
            auto_flow: LaneAutoFlow::Column,
            lane_gap: 0.0,
            tolerance: LaneFlowTolerance::Infinite,
            tolerance_basis: 0.0,
            items: vec![
                LaneItemInput::auto("a", 1, 10.0),
                LaneItemInput::auto("b", 1, 10.0),
                LaneItemInput::auto("c", 1, 10.0),
            ],
        },
    );
}

#[test]
fn lanes_intrinsic_groups_indefinite_items_like_oracle() {
    let facts = oracle_lane_facts(20.0, 50.0);
    let production_facts = production_lane_facts(20.0, 50.0);
    assert_production_lane_intrinsic_matches_oracle(
        ProductionLaneIntrinsicSizingInput {
            axis: ProductionGridAxisKind::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![
                ProductionTrackSizing::AUTO,
                ProductionTrackSizing::AUTO,
                ProductionTrackSizing::AUTO,
            ],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![
                ProductionLaneIntrinsicItem::indefinite("a", 2, production_facts),
                ProductionLaneIntrinsicItem::indefinite(
                    "b",
                    2,
                    ProductionLaneContributionFacts {
                        min_content: 30.0,
                        max_content: 60.0,
                        ..production_facts
                    },
                ),
            ],
        },
        LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![
                LaneIntrinsicItem::indefinite("a", 2, facts),
                LaneIntrinsicItem::indefinite(
                    "b",
                    2,
                    ItemContributionFacts {
                        min_content: 30.0,
                        max_content: 60.0,
                        ..facts
                    },
                ),
            ],
        },
    );
}

#[test]
fn lanes_intrinsic_skips_definite_items_outside_content_sized_tracks() {
    let facts = oracle_lane_facts(80.0, 120.0);
    let production_facts = production_lane_facts(80.0, 120.0);
    assert_production_lane_intrinsic_matches_oracle(
        ProductionLaneIntrinsicSizingInput {
            axis: ProductionGridAxisKind::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![ProductionTrackSizing::AUTO, ProductionTrackSizing::AUTO],
            content_sized_tracks: vec![1],
            items: vec![ProductionLaneIntrinsicItem::definite(
                "a",
                1,
                2,
                production_facts,
            )],
        },
        LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![1],
            items: vec![LaneIntrinsicItem::definite(
                "a",
                support::oracle::grid::TrackSpan::new(1, 2),
                facts,
            )],
        },
    );
}

#[test]
fn lanes_intrinsic_projects_disjoint_content_sized_spans_like_oracle() {
    let facts = ItemContributionFacts {
        automatic_minimum_applies: false,
        min_size: ContributionSize::Definite(100.0),
        ..oracle_lane_facts(120.0, 160.0)
    };
    let production_facts = ProductionLaneContributionFacts {
        automatic_minimum_applies: false,
        min_size: 100.0,
        ..production_lane_facts(120.0, 160.0)
    };
    assert_production_lane_intrinsic_matches_oracle(
        ProductionLaneIntrinsicSizingInput {
            axis: ProductionGridAxisKind::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![
                ProductionTrackSizing::AUTO,
                ProductionTrackSizing::px(20.0),
                ProductionTrackSizing::AUTO,
            ],
            content_sized_tracks: vec![0, 2],
            items: vec![ProductionLaneIntrinsicItem::indefinite(
                "a",
                3,
                production_facts,
            )],
        },
        LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::fixed(20.0), GridTrack::auto()],
            content_sized_tracks: vec![0, 2],
            items: vec![LaneIntrinsicItem::indefinite("a", 3, facts)],
        },
    );
}

#[test]
fn lanes_intrinsic_clamps_oversized_indefinite_spans_like_oracle() {
    let facts = oracle_lane_facts(30.0, 60.0);
    let production_facts = production_lane_facts(30.0, 60.0);
    assert_production_lane_intrinsic_matches_oracle(
        ProductionLaneIntrinsicSizingInput {
            axis: ProductionGridAxisKind::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![ProductionTrackSizing::AUTO, ProductionTrackSizing::AUTO],
            content_sized_tracks: vec![0, 1],
            items: vec![ProductionLaneIntrinsicItem::indefinite(
                "a",
                5,
                production_facts,
            )],
        },
        LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![LaneIntrinsicItem::indefinite("a", 5, facts)],
        },
    );
}

#[test]
fn lanes_intrinsic_preserves_min_content_track_behavior() {
    let facts = ItemContributionFacts {
        automatic_minimum_applies: false,
        min_size: ContributionSize::Definite(12.0),
        ..oracle_lane_facts(100.0, 120.0)
    };
    let production_facts = ProductionLaneContributionFacts {
        automatic_minimum_applies: false,
        min_size: 12.0,
        ..production_lane_facts(100.0, 120.0)
    };
    assert_production_lane_intrinsic_matches_oracle(
        ProductionLaneIntrinsicSizingInput {
            axis: ProductionGridAxisKind::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![ProductionTrackSizing::new(
                MinTrackSizing::MIN_CONTENT,
                MaxTrackSizing::MAX_CONTENT,
            )],
            content_sized_tracks: vec![0],
            items: vec![ProductionLaneIntrinsicItem::indefinite(
                "a",
                1,
                production_facts,
            )],
        },
        LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent)],
            content_sized_tracks: vec![0],
            items: vec![LaneIntrinsicItem::indefinite("a", 1, facts)],
        },
    );
}

#[test]
fn lanes_intrinsic_reports_nested_indefinite_subgrid_unsupported_like_oracle() {
    let production_facts = production_lane_facts(20.0, 50.0);
    let production = production_lane_intrinsic_sizing(ProductionLaneIntrinsicSizingInput {
        axis: ProductionGridAxisKind::Column,
        available: Some(300.0),
        gap: 10.0,
        tracks: vec![
            ProductionTrackSizing::AUTO,
            ProductionTrackSizing::AUTO,
            ProductionTrackSizing::AUTO,
        ],
        content_sized_tracks: vec![0, 1, 2],
        items: vec![ProductionLaneIntrinsicItem::nested_indefinite_subgrid(
            "subgrid-child",
            2,
            production_facts,
        )],
    });

    assert!(production.is_err());
}

#[test]
fn lanes_content_size_contributes_to_indefinite_container_size() {
    let expected_columns = TrackSizingSlice::indefinite_columns(0.0)
        .track(GridTrack::fixed(40.0))
        .solve();
    let expected_rows = TrackSizingSlice::indefinite_rows(8.0)
        .track(GridTrack::fixed(10.0))
        .solve();

    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(40.0, 0.0))
        .root_size(Size::new(Dimension::px(40.0), Dimension::Auto))
        .columns(vec![TrackComponent::px(40.0)])
        .rows(vec![TrackComponent::px(10.0)])
        .gap(Size::new(0.0, 8.0))
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                .measurement(Size::new(20.0, 30.0))
                .expect_layout(Point::new(0.0, 0.0), Size::new(20.0, 30.0)),
        )
        .node(
            GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                .measurement(Size::new(20.0, 50.0))
                .expect_layout(Point::new(0.0, 38.0), Size::new(20.0, 50.0)),
        )
        .assert_layout_size(Size::new(40.0, 88.0));
}

#[test]
fn lanes_child_measurement_uses_resolved_grid_axis_span_size() {
    let expected_columns = TrackSizingSlice::definite_columns(100.0, 0.0)
        .track(GridTrack::fixed(100.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(10.0, 0.0)
        .track(GridTrack::fixed(10.0))
        .solve();
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                grid_template_columns: vec![TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 60.0),
                Size::new(100.0, 60.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .known(Size::new(Some(100.0), None))
            .parent(Size::new(Some(100.0), None))
            .available(Size::new(
                Available::Definite(100.0),
                Available::MAX_CONTENT,
            )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 60.0),
                Size::new(100.0, 60.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .known(Size::new(Some(100.0), None))
            .parent(Size::new(Some(100.0), Some(0.0)))
            .available(Size::new(
                Available::Definite(100.0),
                Available::MAX_CONTENT,
            )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 60.0),
                Size::new(100.0, 60.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .known(Size::new(Some(100.0), Some(60.0)))
            .parent(Size::new(Some(100.0), Some(60.0)))
            .available(Size::new(
                Available::Definite(100.0),
                Available::Definite(60.0),
            )),
        );

    let output = surgeist::layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(100.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(100.0, 100.0));
    let child = tree.layout(2).expect("lane child layout must be recorded");
    assert_eq!(child.location, Point::new(0.0, 0.0));
    assert_eq!(child.size, Size::new(100.0, 60.0));
    let compute_size_inputs = tree
        .inputs(2)
        .iter()
        .filter(|input| input.run_mode == RunMode::ComputeSize)
        .collect::<Vec<_>>();
    assert!(
        compute_size_inputs.iter().any(|input| {
            input.known == Size::new(Some(100.0), None)
                && input.parent == Size::new(Some(100.0), None)
                && input.available == Size::new(Available::Definite(100.0), Available::MAX_CONTENT)
        }),
        "lane placement should measure child against resolved grid-axis span: {compute_size_inputs:#?}"
    );

    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(100.0, 100.0))
        .columns(vec![TrackComponent::px(100.0)])
        .rows(vec![TrackComponent::px(10.0)])
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                .measurement(Size::new(100.0, 60.0))
                .expect_layout(Point::new(0.0, 0.0), Size::new(100.0, 60.0)),
        )
        .assert_layout();
}

#[test]
fn lanes_auto_child_measurement_uses_final_auto_placement_span() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3, 4])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(Dimension::px(140.0), Dimension::px(140.0)),
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(100.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                grid_flow_tolerance: GridFlowTolerance::Length(Length::px(0.0)),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .style(3, NodeInput::default())
        .style(4, NodeInput::default());

    tree = tree
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(40.0, 100.0),
                Size::new(40.0, 100.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 10.0),
                Size::new(100.0, 10.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(40.0, 100.0),
                Size::new(40.0, 100.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .available(Size::new(Available::Definite(40.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 10.0),
                Size::new(100.0, 10.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .available(Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(40.0, 100.0),
                Size::new(40.0, 100.0),
            ))
            .available(Size::new(Available::Definite(40.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 10.0),
                Size::new(100.0, 10.0),
            ))
            .available(Size::new(
                Available::Definite(100.0),
                Available::MAX_CONTENT,
            )),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 10.0),
                Size::new(100.0, 10.0),
            ))
            .available(Size::new(
                Available::Definite(100.0),
                Available::MAX_CONTENT,
            )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(40.0, 100.0),
                Size::new(40.0, 100.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .available(Size::new(
                Available::Definite(40.0),
                Available::Definite(100.0),
            )),
        )
        .measure_when(
            3,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 10.0),
                Size::new(100.0, 10.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .available(Size::new(
                Available::Definite(100.0),
                Available::Definite(10.0),
            )),
        )
        .measure_when(
            4,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(100.0, 10.0),
                Size::new(100.0, 10.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .available(Size::new(
                Available::Definite(100.0),
                Available::Definite(10.0),
            )),
        );

    surgeist::layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(140.0), Some(140.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let first = tree.layout(2).expect("first child layout");
    let second = tree.layout(3).expect("second child layout");
    let third = tree.layout(4).expect("third child layout");
    assert_eq!(first.location, Point::new(0.0, 0.0));
    assert_eq!(first.size, Size::new(40.0, 100.0));
    assert_eq!(second.location, Point::new(40.0, 0.0));
    assert_eq!(second.size, Size::new(100.0, 10.0));
    assert_eq!(third.location, Point::new(40.0, 10.0));
    assert_eq!(third.size, Size::new(100.0, 10.0));

    let third_compute_size_inputs = tree
        .inputs(4)
        .iter()
        .filter(|input| input.run_mode == RunMode::ComputeSize)
        .collect::<Vec<_>>();
    assert!(
        third_compute_size_inputs
            .iter()
            .any(|input| input.available.width == Available::Definite(100.0)),
        "third auto lane item should be measured against its final 100px column: {third_compute_size_inputs:#?}"
    );
}

#[test]
fn lanes_spanning_child_measurement_uses_distributed_grid_axis_gap() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .style(
            1,
            NodeInput {
                display: Display::GridLanes,
                size: Size::new(Dimension::px(120.0), Dimension::px(120.0)),
                grid_template_columns: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(10.0)],
                justify_content: Some(AlignContent::SpaceBetween),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: surgeist::layout::GridPlacement::span(2),
                ..NodeInput::default()
            },
        );
    tree = tree
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(80.0, 40.0),
                Size::new(80.0, 40.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .available(Size::new(Available::Definite(80.0), Available::MAX_CONTENT)),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(120.0, 40.0),
                Size::new(120.0, 40.0),
            ))
            .run_mode(RunMode::ComputeSize)
            .available(Size::new(
                Available::Definite(120.0),
                Available::MAX_CONTENT,
            )),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(120.0, 40.0),
                Size::new(120.0, 40.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .available(Size::new(
                Available::Definite(120.0),
                Available::Definite(40.0),
            )),
        );

    surgeist::layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(120.0), Some(120.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    let child = tree.layout(2).expect("spanning lane child layout");
    assert_eq!(child.location, Point::new(0.0, 0.0));
    assert_eq!(child.size, Size::new(120.0, 40.0));
    let compute_size_inputs = tree
        .inputs(2)
        .iter()
        .filter(|input| input.run_mode == RunMode::ComputeSize)
        .collect::<Vec<_>>();
    assert!(
        compute_size_inputs
            .iter()
            .any(|input| input.available.width == Available::Definite(120.0)),
        "spanning lane child should be measured against distributed 120px grid-axis span: {compute_size_inputs:#?}"
    );
}

#[test]
fn lanes_absolute_child_uses_grid_absolute_layout() {
    let expected_columns = TrackSizingSlice::definite_columns(100.0, 0.0)
        .track(GridTrack::fixed(100.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(40.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .solve();

    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(100.0, 40.0))
        .columns(vec![TrackComponent::px(100.0)])
        .rows(vec![TrackComponent::px(40.0)])
        .expected_tracks(expected_columns, expected_rows)
        .node(
            GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                .position(Position::Absolute)
                .size(Size::new(Dimension::px(24.0), Dimension::px(12.0)))
                .expect_layout(Point::new(0.0, 0.0), Size::new(24.0, 12.0)),
        )
        .assert_layout();
}

#[test]
fn lanes_indefinite_nested_subgrid_does_not_contribute_as_ordinary_lane_item() {
    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(0.0, 10.0))
        .root_size(Size::new(Dimension::Auto, Dimension::px(10.0)))
        .columns(vec![TrackComponent::AUTO])
        .rows(vec![TrackComponent::px(10.0)])
        .expected_tracks(
            TrackSizingSlice::indefinite_columns(0.0)
                .track(GridTrack::auto())
                .solve(),
            TrackSizingSlice::definite_rows(10.0, 0.0)
                .track(GridTrack::fixed(10.0))
                .solve(),
        )
        .node(
            GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                .display(surgeist::layout::Display::GridLanes)
                .columns(vec![TrackComponent::Subgrid(
                    surgeist::layout::SubgridTrack {
                        name_components: Vec::new(),
                    },
                )])
                .rows(vec![TrackComponent::px(10.0)])
                .measurement(Size::new(90.0, 10.0))
                .expect_layout(Point::new(0.0, 0.0), Size::new(0.0, 10.0)),
        )
        .assert_layout_size(Size::new(0.0, 10.0));
}

#[test]
fn lanes_child_subgrid_inherits_grid_axis_tracks() {
    let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(120.0, 30.0))
        .columns(vec![TrackComponent::px(40.0), TrackComponent::px(80.0)])
        .rows(vec![TrackComponent::px(30.0)])
        .expected_tracks(expected_columns, expected_rows)
        .auto_flow(GridAutoFlow::Row)
        .node(
            GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 2, 1), 2, 1)
                .display(surgeist::layout::Display::Grid)
                .columns(vec![TrackComponent::Subgrid(
                    surgeist::layout::SubgridTrack {
                        name_components: Vec::new(),
                    },
                )])
                .rows(vec![TrackComponent::px(30.0)])
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .measurement(Size::new(12.0, 10.0))
                        .expect_layout(Point::new(40.0, 0.0), Size::new(12.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn lanes_column_flow_child_subgrid_inherits_row_axis_tracks() {
    let expected_columns = TrackSizingSlice::definite_columns(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(120.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .solve();

    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(30.0, 120.0))
        .columns(vec![TrackComponent::px(30.0)])
        .rows(vec![TrackComponent::px(40.0), TrackComponent::px(80.0)])
        .expected_tracks(expected_columns, expected_rows)
        .auto_flow(GridAutoFlow::Column)
        .node(
            GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 1, 2), 1, 2)
                .display(surgeist::layout::Display::Grid)
                .columns(vec![TrackComponent::px(30.0)])
                .rows(vec![TrackComponent::Subgrid(
                    surgeist::layout::SubgridTrack {
                        name_components: Vec::new(),
                    },
                )])
                .child(
                    GridLayoutNode::item(GridArea::new(1, 2, 1, 1))
                        .measurement(Size::new(12.0, 10.0))
                        .expect_layout(Point::new(0.0, 40.0), Size::new(12.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn lanes_child_subgrid_uses_report_matching_child_order_after_skipped_siblings() {
    let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(80.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(30.0, 0.0)
        .track(GridTrack::fixed(30.0))
        .solve();

    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(120.0, 30.0))
        .columns(vec![TrackComponent::px(40.0), TrackComponent::px(80.0)])
        .rows(vec![TrackComponent::px(30.0)])
        .expected_tracks(expected_columns, expected_rows)
        .auto_flow(GridAutoFlow::Row)
        .node(
            GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                .display(surgeist::layout::Display::None)
                .expect_layout(Point::new(0.0, 0.0), Size::ZERO),
        )
        .node(
            GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                .position(Position::Absolute)
                .size(Size::new(Dimension::px(8.0), Dimension::px(6.0)))
                .expect_layout(Point::new(0.0, 0.0), Size::new(8.0, 6.0)),
        )
        .node(
            GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 2, 1), 2, 1)
                .display(surgeist::layout::Display::Grid)
                .columns(vec![TrackComponent::Subgrid(
                    surgeist::layout::SubgridTrack {
                        name_components: Vec::new(),
                    },
                )])
                .rows(vec![TrackComponent::px(30.0)])
                .child(
                    GridLayoutNode::item(GridArea::new(2, 1, 1, 1))
                        .measurement(Size::new(12.0, 10.0))
                        .expect_layout(Point::new(40.0, 0.0), Size::new(12.0, 10.0)),
                ),
        )
        .assert_layout();
}

#[test]
fn lanes_definite_lane_axis_container_lays_out_children_at_lane_offsets() {
    let expected_columns = TrackSizingSlice::definite_columns(120.0, 0.0)
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(40.0))
        .track(GridTrack::fixed(40.0))
        .solve();
    let expected_rows = TrackSizingSlice::definite_rows(90.0, 6.0)
        .track(GridTrack::fixed(10.0))
        .solve();

    GridLayoutComparison::new()
        .root_display(surgeist::layout::Display::GridLanes)
        .container(Size::new(120.0, 90.0))
        .columns(vec![
            TrackComponent::px(40.0),
            TrackComponent::px(40.0),
            TrackComponent::px(40.0),
        ])
        .rows(vec![TrackComponent::px(10.0)])
        .gap(Size::new(0.0, 6.0))
        .expected_tracks(expected_columns, expected_rows)
        .auto_flow(GridAutoFlow::Row)
        .node(
            GridLayoutNode::auto_spanning_item(GridArea::new(1, 1, 2, 1), 2, 1)
                .measurement(Size::new(20.0, 30.0))
                .expect_layout(Point::new(0.0, 0.0), Size::new(20.0, 30.0)),
        )
        .node(
            GridLayoutNode::auto_item(GridArea::new(3, 1, 1, 1))
                .measurement(Size::new(20.0, 20.0))
                .expect_layout(Point::new(80.0, 0.0), Size::new(20.0, 20.0)),
        )
        .node(
            GridLayoutNode::auto_item(GridArea::new(1, 1, 1, 1))
                .measurement(Size::new(20.0, 15.0))
                .expect_layout(Point::new(0.0, 36.0), Size::new(20.0, 15.0)),
        )
        .assert_layout();
}

fn assert_production_lane_placement_matches_oracle(
    production_input: ProductionLanePlacementInput<&'static str>,
    oracle_input: LanePlacementInput,
) {
    let production = production_place_lanes(production_input).unwrap();
    let oracle = support::oracle::grid::place_lanes(oracle_input).unwrap();

    assert_eq!(
        production_grid_axis(production.lane_axis),
        oracle.lane_axis,
        "lane axis"
    );
    assert_eq!(
        production_grid_axis(production.grid_axis),
        oracle.grid_axis,
        "grid axis"
    );
    assert_eq!(production.content_size, oracle.content_size, "content size");
    assert_eq!(production.final_cursor, oracle.final_cursor, "final cursor");
    assert_eq!(
        production.running_positions_after_each_item, oracle.running_positions_after_each_item,
        "running positions"
    );
    assert_eq!(
        production
            .item_offsets
            .iter()
            .map(|item| (
                item.item,
                item.grid_axis_start,
                item.grid_axis_span,
                item.offset
            ))
            .collect::<Vec<_>>(),
        oracle
            .item_offsets
            .iter()
            .map(|item| (
                item.id,
                item.grid_axis_start,
                item.grid_axis_span,
                item.offset
            ))
            .collect::<Vec<_>>(),
        "item offsets"
    );
}

fn assert_production_lane_intrinsic_matches_oracle(
    production_input: ProductionLaneIntrinsicSizingInput,
    oracle_input: LaneIntrinsicSizingInput,
) {
    let production = production_lane_intrinsic_sizing(production_input).unwrap();
    let oracle = support::oracle::grid::lane_intrinsic_sizing(oracle_input).unwrap();

    assert_eq!(
        production
            .definite_items
            .iter()
            .map(|item| (item.id, item.span.start, item.span.end))
            .collect::<Vec<_>>(),
        oracle
            .definite_items
            .iter()
            .map(|item| (item.id, item.span.start, item.span.end))
            .collect::<Vec<_>>(),
        "definite items"
    );
    assert_eq!(
        production
            .indefinite_groups
            .iter()
            .map(|group| {
                (
                    group.span,
                    group.max_min_content,
                    group.max_max_content,
                    group.max_min_size,
                    group.item_ids.clone(),
                )
            })
            .collect::<Vec<_>>(),
        oracle
            .indefinite_groups
            .iter()
            .map(|group| {
                (
                    group.span,
                    group.max_min_content,
                    group.max_max_content,
                    group.max_min_size,
                    group.item_ids.clone(),
                )
            })
            .collect::<Vec<_>>(),
        "indefinite groups"
    );
    assert_eq!(
        production
            .converted_indefinite_items
            .iter()
            .map(|item| (item.id, item.span.start, item.span.end))
            .collect::<Vec<_>>(),
        oracle
            .converted_indefinite_items
            .iter()
            .map(|item| (item.id, item.span.start, item.span.end))
            .collect::<Vec<_>>(),
        "converted indefinite items"
    );
    assert_eq!(
        production
            .final_track_sizes
            .iter()
            .map(|size| (size * 1000.0).round() / 1000.0)
            .collect::<Vec<_>>(),
        oracle
            .final_track_report
            .final_tracks
            .iter()
            .map(|track| (track.size * 1000.0).round() / 1000.0)
            .collect::<Vec<_>>(),
        "final track sizes"
    );
}

fn oracle_lane_facts(min_content: f32, max_content: f32) -> ItemContributionFacts {
    ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content,
        max_content,
        preferred: ContributionSize::Auto,
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Infinite,
        margin_before: 0.0,
        margin_after: 0.0,
        automatic_minimum_applies: true,
    }
}

fn production_lane_facts(min_content: f32, max_content: f32) -> ProductionLaneContributionFacts {
    ProductionLaneContributionFacts {
        min_content,
        max_content,
        min_size: min_content,
        automatic_minimum_applies: true,
    }
}

fn production_grid_axis(axis: ProductionGridAxisKind) -> GridAxis {
    match axis {
        ProductionGridAxisKind::Column => GridAxis::Column,
        ProductionGridAxisKind::Row => GridAxis::Row,
    }
}

fn production_auto_lane_item(
    item: &'static str,
    grid_axis_span: usize,
    lane_axis_margin_box: f32,
) -> ProductionLaneItem<&'static str> {
    ProductionLaneItem {
        item,
        grid_axis_span,
        definite_grid_axis_start: None,
        lane_axis_margin_box,
    }
}

fn production_definite_lane_item(
    item: &'static str,
    grid_axis_start: usize,
    grid_axis_span: usize,
    lane_axis_margin_box: f32,
) -> ProductionLaneItem<&'static str> {
    ProductionLaneItem {
        item,
        grid_axis_span,
        definite_grid_axis_start: Some(grid_axis_start),
        lane_axis_margin_box,
    }
}
