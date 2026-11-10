//! Invalid grid line names and template areas are rejected by `compute_layout`.
//!
//! CSS Grid Layout Module Level 1 section 7.2 forbids `span` and `auto` as line
//! names, requires a repeat count of at least one, and allows one automatic
//! repeat that holds a fixed track size and is not nested in another repeat.
//! Section 7.3 requires template-area rows of equal length and rectangular
//! named areas. Level 2 allows one `auto-fill` per subgrid line-name list.
//! Every shape must produce `InvalidInput`; the controls are valid input.

use super::fixtures::{fri08_c01_placement_output, fri08_c01_placement_request};
use super::*;

#[derive(Clone, Copy, Debug)]
enum Axis {
    Columns,
    Rows,
}

const AXES: [Axis; 2] = [Axis::Columns, Axis::Rows];

fn px<S: LayoutScalar>(value: f64) -> TrackComponentOf<S> {
    TrackComponentOf::px(S::from_f64(value))
}

fn track_px<S: LayoutScalar>(value: f64) -> TrackSizingOf<S> {
    TrackSizingOf::px(S::from_f64(value))
}

fn names<S: LayoutScalar>(names: &[&str]) -> TrackComponentOf<S> {
    TrackComponentOf::line_names(names.iter().copied())
}

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

fn bare(name: &str) -> RawGridPlacement {
    RawGridPlacement::new(
        RawGridLine::BareIdent(name.to_string()),
        RawGridLine::BareIdent(name.to_string()),
    )
}

fn columns<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![px(40.0), px(60.0)]
}

fn rows<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![px(20.0), px(20.0)]
}

// A 100px by 40px grid with 40px and 60px columns and two 20px rows, in which
// the tracks of `axis` are replaced by `tracks`.
fn grid_with<S: LayoutScalar>(axis: Axis, tracks: Vec<TrackComponentOf<S>>) -> NodeInputOf<S> {
    let (grid_template_columns, grid_template_rows) = match axis {
        Axis::Columns => (tracks, rows()),
        Axis::Rows => (columns(), tracks),
    };
    NodeInputOf {
        display: Display::Grid,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(100.0)),
            PreferredSizeOf::px(S::from_f64(40.0)),
        ),
        grid_template_columns,
        grid_template_rows,
        ..NodeInputOf::default()
    }
}

fn placed_in<S: LayoutScalar>(axis: Axis, placement: RawGridPlacement) -> NodeInputOf<S> {
    match axis {
        Axis::Columns => NodeInputOf {
            raw_grid_column: placement,
            ..NodeInputOf::default()
        },
        Axis::Rows => NodeInputOf {
            raw_grid_row: placement,
            ..NodeInputOf::default()
        },
    }
}

// Node 1 is the grid and node 2 is its only item.
fn single_item_tree<S: LayoutScalar>(
    grid: NodeInputOf<S>,
    item: NodeInputOf<S>,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(1, grid)
        .style(2, item)
}

// Node 1 is the grid, node 2 a subgrid spanning both tracks of each axis, and
// node 3 an item of the subgrid. The subgrid track lists of `axis` carry
// `name_components`; the other axis is an empty subgrid.
fn subgrid_tree<S: LayoutScalar>(
    axis: Axis,
    name_components: Vec<SubgridLineNameComponent>,
    item: NodeInputOf<S>,
) -> PublicLayoutTreeOf<S> {
    let named = vec![TrackComponentOf::Subgrid(SubgridTrack { name_components })];
    let empty = vec![TrackComponentOf::Subgrid(SubgridTrack {
        name_components: Vec::new(),
    })];
    let (grid_template_columns, grid_template_rows) = match axis {
        Axis::Columns => (named, empty),
        Axis::Rows => (empty, named),
    };
    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::px(S::from_f64(40.0)),
                ),
                grid_template_columns: columns(),
                grid_template_rows: rows(),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_span(2).unwrap(),
                grid_row: GridPlacement::try_span(2).unwrap(),
                grid_template_columns,
                grid_template_rows,
                ..NodeInputOf::default()
            },
        )
        .style(3, item)
}

fn grid_axis(axis: Axis) -> GridAxisKind {
    match axis {
        Axis::Columns => GridAxisKind::Column,
        Axis::Rows => GridAxisKind::Row,
    }
}

// Layout must return `InvalidInput(GridNames(expected))` at the `site` node;
// any other result is described.
fn rejection<S: LayoutScalar>(
    label: &str,
    tree: &PublicLayoutTreeOf<S>,
    site: u32,
    expected: NamedGridErrorReport,
) -> Option<String> {
    match compute_layout(tree, 1, fri08_c01_placement_request()) {
        Err(error) => {
            let actual_site = error.site();
            match error.kind() {
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::GridNames(actual))
                    if *actual == expected && actual_site == LayoutErrorSiteOf::Node(site) =>
                {
                    None
                }
                other => Some(format!(
                    "{label}: error {other:?} at {actual_site:?} instead of GridNames({expected:?}) at Node({site})"
                )),
            }
        }
        Ok(_) => Some(format!("{label}: layout succeeded")),
    }
}

fn collect(results: impl IntoIterator<Item = Option<String>>) -> Vec<String> {
    results.into_iter().flatten().collect()
}

macro_rules! lane_tests {
    ($case:ident, $f32_test:ident, $f64_test:ident) => {
        #[test]
        fn $f32_test() {
            let failures = $case::<f32>();
            assert!(failures.is_empty(), "{}", failures.join("\n"));
        }

        #[test]
        fn $f64_test() {
            let failures = $case::<f64>();
            assert!(failures.is_empty(), "{}", failures.join("\n"));
        }
    };
}

fn reserved_names_in_track_lists<S: LayoutScalar>() -> Vec<String> {
    let mut results = Vec::new();
    for axis in AXES {
        for reserved in ["span", "auto"] {
            let tracks = vec![
                names::<S>(&["main-start"]),
                px(10.0),
                names(&["main-end", reserved]),
                px(10.0),
            ];
            let tree =
                single_item_tree::<S>(grid_with(axis, tracks), placed_in(axis, bare("main")));
            results.push(rejection(
                &format!("{axis:?} line name {reserved}"),
                &tree,
                1,
                NamedGridErrorReport::ReservedLineName {
                    name: reserved.to_string(),
                },
            ));
        }
    }
    collect(results)
}

lane_tests!(
    reserved_names_in_track_lists,
    reserved_line_name_in_track_list_is_invalid_input_f32,
    reserved_line_name_in_track_list_is_invalid_input_f64
);

fn reserved_names_in_subgrid_lists<S: LayoutScalar>() -> Vec<String> {
    let mut results = Vec::new();
    for axis in AXES {
        for reserved in ["span", "auto"] {
            let shapes = [
                (
                    "line names",
                    vec![SubgridLineNameComponent::LineNames(owned(&[reserved]))],
                ),
                (
                    "counted repeat",
                    vec![SubgridLineNameComponent::Repeat {
                        count: SubgridLineNameRepeatCount::Count(1),
                        line_name_sets: vec![owned(&["a"]), owned(&[reserved])],
                    }],
                ),
                (
                    "auto-fill repeat",
                    vec![SubgridLineNameComponent::Repeat {
                        count: SubgridLineNameRepeatCount::AutoFill,
                        line_name_sets: vec![owned(&[reserved])],
                    }],
                ),
            ];
            for (shape, components) in shapes {
                let tree = subgrid_tree::<S>(axis, components, NodeInputOf::default());
                results.push(rejection(
                    &format!("{axis:?} subgrid {shape} {reserved}"),
                    &tree,
                    2,
                    NamedGridErrorReport::ReservedLineName {
                        name: reserved.to_string(),
                    },
                ));
            }
        }
    }
    collect(results)
}

lane_tests!(
    reserved_names_in_subgrid_lists,
    reserved_line_name_in_subgrid_line_names_is_invalid_input_f32,
    reserved_line_name_in_subgrid_line_names_is_invalid_input_f64
);

fn zero_subgrid_repeat_count<S: LayoutScalar>() -> Vec<String> {
    collect(AXES.map(|axis| {
        let components = vec![SubgridLineNameComponent::Repeat {
            count: SubgridLineNameRepeatCount::Count(0),
            line_name_sets: vec![owned(&["a"])],
        }];
        rejection(
            &format!("{axis:?} subgrid repeat count 0"),
            &subgrid_tree::<S>(axis, components, NodeInputOf::default()),
            2,
            NamedGridErrorReport::ZeroRepeat {
                axis: grid_axis(axis),
            },
        )
    }))
}

lane_tests!(
    zero_subgrid_repeat_count,
    zero_subgrid_line_name_repeat_count_is_invalid_input_f32,
    zero_subgrid_line_name_repeat_count_is_invalid_input_f64
);

fn repeated_subgrid_auto_fill<S: LayoutScalar>() -> Vec<String> {
    collect(AXES.map(|axis| {
        let auto_fill = |name: &str| SubgridLineNameComponent::Repeat {
            count: SubgridLineNameRepeatCount::AutoFill,
            line_name_sets: vec![owned(&[name])],
        };
        rejection(
            &format!("{axis:?} two subgrid auto-fill repeats"),
            &subgrid_tree::<S>(
                axis,
                vec![auto_fill("a"), auto_fill("b")],
                NodeInputOf::default(),
            ),
            2,
            NamedGridErrorReport::MultipleAutoFillRepeats {
                axis: grid_axis(axis),
            },
        )
    }))
}

lane_tests!(
    repeated_subgrid_auto_fill,
    repeated_auto_fill_in_subgrid_line_names_is_invalid_input_f32,
    repeated_auto_fill_in_subgrid_line_names_is_invalid_input_f64
);

fn repeat_of<S: LayoutScalar>(
    result: Result<TrackRepetitionOf<S>, TrackRepetitionError>,
) -> TrackComponentOf<S> {
    TrackComponentOf::Repeat(result.expect("constructible repetition"))
}

fn repeated_automatic_track_repeats<S: LayoutScalar>() -> Vec<String> {
    let mut results = Vec::new();
    for axis in AXES {
        let auto_fill = || repeat_of(TrackRepetitionOf::auto_fill(vec![track_px::<S>(20.0)]));
        let auto_fit = || repeat_of(TrackRepetitionOf::auto_fit(vec![track_px::<S>(20.0)]));
        let shapes = [
            ("auto-fill then auto-fill", vec![auto_fill(), auto_fill()]),
            ("auto-fit then auto-fit", vec![auto_fit(), auto_fit()]),
            ("auto-fill then auto-fit", vec![auto_fill(), auto_fit()]),
            (
                "auto-fill, track, auto-fill",
                vec![auto_fill(), px(10.0), auto_fill()],
            ),
        ];
        for (shape, tracks) in shapes {
            results.push(rejection(
                &format!("{axis:?} {shape}"),
                &single_item_tree(grid_with(axis, tracks), NodeInputOf::default()),
                1,
                NamedGridErrorReport::UnresolvedAutoRepeatNames {
                    axis: grid_axis(axis),
                },
            ));
        }
    }
    collect(results)
}

lane_tests!(
    repeated_automatic_track_repeats,
    more_than_one_automatic_repeat_in_track_list_is_invalid_input_f32,
    more_than_one_automatic_repeat_in_track_list_is_invalid_input_f64
);

fn nested_automatic_track_repeats<S: LayoutScalar>() -> Vec<String> {
    let mut results = Vec::new();
    for axis in AXES {
        for (shape, inner) in [
            (
                "auto-fill",
                TrackRepetitionOf::auto_fill(vec![track_px::<S>(20.0)]),
            ),
            (
                "auto-fit",
                TrackRepetitionOf::auto_fit(vec![track_px::<S>(20.0)]),
            ),
        ] {
            let outer = repeat_of(TrackRepetitionOf::count_components(
                2,
                vec![repeat_of(inner)],
            ));
            results.push(rejection(
                &format!("{axis:?} {shape} nested in counted repeat"),
                &single_item_tree(grid_with(axis, vec![outer]), NodeInputOf::default()),
                1,
                NamedGridErrorReport::UnresolvedAutoRepeatNames {
                    axis: grid_axis(axis),
                },
            ));
        }
    }
    collect(results)
}

lane_tests!(
    nested_automatic_track_repeats,
    automatic_repeat_nested_in_counted_repeat_is_invalid_input_f32,
    automatic_repeat_nested_in_counted_repeat_is_invalid_input_f64
);

fn automatic_track_repeats_without_track_size<S: LayoutScalar>() -> Vec<String> {
    let mut results = Vec::new();
    for axis in AXES {
        for (shape, repetition) in [
            (
                "auto-fill",
                TrackRepetitionOf::auto_fill_components(vec![names::<S>(&["a"])]),
            ),
            (
                "auto-fit",
                TrackRepetitionOf::auto_fit_components(vec![names::<S>(&["a"])]),
            ),
        ] {
            let tracks = vec![px(10.0), repeat_of(repetition)];
            results.push(rejection(
                &format!("{axis:?} {shape} with line names only"),
                &single_item_tree(grid_with(axis, tracks), NodeInputOf::default()),
                1,
                NamedGridErrorReport::UnresolvedAutoRepeatNames {
                    axis: grid_axis(axis),
                },
            ));
        }
    }
    collect(results)
}

lane_tests!(
    automatic_track_repeats_without_track_size,
    automatic_repeat_without_track_size_is_invalid_input_f32,
    automatic_repeat_without_track_size_is_invalid_input_f64
);

fn area_row(cells: &[Option<&str>]) -> GridTemplateAreaRow {
    GridTemplateAreaRow {
        cells: cells.iter().map(|cell| cell.map(str::to_string)).collect(),
    }
}

fn grid_with_areas<S: LayoutScalar>(rows: Vec<GridTemplateAreaRow>) -> NodeInputOf<S> {
    NodeInputOf {
        grid_template_areas: GridTemplateAreas { rows },
        ..grid_with(Axis::Columns, columns())
    }
}

fn unequal_template_area_rows<S: LayoutScalar>() -> Vec<String> {
    let shapes = [
        (
            "short second row",
            vec![area_row(&[Some("a"), Some("a")]), area_row(&[Some("b")])],
            (2, 2, 1),
        ),
        (
            "long second row",
            vec![area_row(&[Some("a")]), area_row(&[Some("b"), Some("b")])],
            (2, 1, 2),
        ),
        (
            "empty second row",
            vec![area_row(&[Some("a"), Some("a")]), area_row(&[])],
            (2, 2, 0),
        ),
        ("empty first row", vec![area_row(&[])], (1, 1, 0)),
    ];
    collect(
        shapes
            .into_iter()
            .map(|(shape, rows, (row, expected, actual))| {
                rejection(
                    &format!("areas with {shape}"),
                    &single_item_tree(
                        grid_with_areas::<S>(rows),
                        placed_in(Axis::Columns, bare("a")),
                    ),
                    1,
                    NamedGridErrorReport::TemplateAreaRowLengthMismatch {
                        row,
                        expected,
                        actual,
                    },
                )
            }),
    )
}

lane_tests!(
    unequal_template_area_rows,
    unequal_template_area_rows_are_invalid_input_f32,
    unequal_template_area_rows_are_invalid_input_f64
);

fn non_rectangular_template_areas<S: LayoutScalar>() -> Vec<String> {
    let shapes = [
        (
            "L-shaped area",
            vec![
                area_row(&[Some("a"), Some("a")]),
                area_row(&[Some("a"), None]),
            ],
        ),
        (
            "diagonal area",
            vec![area_row(&[Some("a"), None]), area_row(&[None, Some("a")])],
        ),
        (
            "interleaved areas",
            vec![
                area_row(&[Some("a"), Some("b")]),
                area_row(&[Some("b"), Some("a")]),
            ],
        ),
        (
            "split area",
            vec![area_row(&[Some("a"), Some("b"), Some("a")])],
        ),
    ];
    collect(shapes.into_iter().map(|(shape, rows)| {
        rejection(
            &format!("areas with {shape}"),
            &single_item_tree(
                grid_with_areas::<S>(rows),
                placed_in(Axis::Columns, bare("a")),
            ),
            1,
            NamedGridErrorReport::NonRectangularTemplateArea {
                name: "a".to_string(),
            },
        )
    }))
}

lane_tests!(
    non_rectangular_template_areas,
    non_rectangular_template_area_is_invalid_input_f32,
    non_rectangular_template_area_is_invalid_input_f64
);

// Controls: valid input still lays out and places by name.

fn placement_of<S: LayoutScalar>(
    label: &str,
    tree: &PublicLayoutTreeOf<S>,
    probe: u32,
    expected: (f64, f64),
) -> Option<String> {
    match compute_layout(tree, 1, fri08_c01_placement_request()) {
        Err(error) => Some(format!("{label}: layout failed with {:?}", error.kind())),
        Ok(batch) => {
            let output = fri08_c01_placement_output(&batch, probe);
            let actual = (output.location.x.to_f64(), output.size.width.to_f64());
            (actual != expected)
                .then(|| format!("{label}: x and width {actual:?} instead of {expected:?}"))
        }
    }
}

fn valid_named_lines_place_items<S: LayoutScalar>() -> Vec<String> {
    let tracks = vec![
        names::<S>(&["main-start"]),
        px(40.0),
        names(&["main-end"]),
        px(60.0),
    ];
    let tree = single_item_tree(
        grid_with(Axis::Columns, tracks),
        placed_in(Axis::Columns, bare("main")),
    );
    collect([placement_of("named columns", &tree, 2, (0.0, 40.0))])
}

lane_tests!(
    valid_named_lines_place_items,
    valid_named_lines_place_item_between_them_f32,
    valid_named_lines_place_item_between_them_f64
);

fn valid_template_areas_extend_explicit_grid<S: LayoutScalar>() -> Vec<String> {
    let grid = NodeInputOf {
        display: Display::Grid,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(120.0)),
            PreferredSizeOf::px(S::from_f64(20.0)),
        ),
        grid_auto_columns: vec![px(40.0)],
        grid_auto_rows: vec![px(20.0)],
        grid_template_areas: GridTemplateAreas {
            rows: vec![area_row(&[Some("main"), Some("main"), Some("main")])],
        },
        ..NodeInputOf::default()
    };
    let item = NodeInputOf {
        raw_grid_column: bare("main"),
        raw_grid_row: bare("main"),
        ..NodeInputOf::default()
    };
    collect([placement_of(
        "areas beyond the explicit tracks",
        &single_item_tree(grid, item),
        2,
        (0.0, 120.0),
    )])
}

lane_tests!(
    valid_template_areas_extend_explicit_grid,
    valid_template_area_spans_implicit_tracks_f32,
    valid_template_area_spans_implicit_tracks_f64
);

fn valid_subgrid_names<S: LayoutScalar>() -> Vec<String> {
    let mut results = Vec::new();
    let parent_tracks = vec![
        names::<S>(&["main-start"]),
        px(40.0),
        names(&["main-end"]),
        px(60.0),
    ];
    let mut tree = subgrid_tree::<S>(
        Axis::Columns,
        Vec::new(),
        placed_in(Axis::Columns, bare("main")),
    );
    tree = tree.style(1, grid_with(Axis::Columns, parent_tracks));
    results.push(placement_of("inherited names", &tree, 3, (0.0, 40.0)));

    let local = vec![
        SubgridLineNameComponent::LineNames(owned(&["loc-start"])),
        SubgridLineNameComponent::LineNames(owned(&["loc-end"])),
    ];
    let tree = subgrid_tree::<S>(Axis::Columns, local, placed_in(Axis::Columns, bare("loc")));
    results.push(placement_of("local names", &tree, 3, (0.0, 40.0)));
    collect(results)
}

lane_tests!(
    valid_subgrid_names,
    valid_subgrid_names_place_items_f32,
    valid_subgrid_names_place_items_f64
);
