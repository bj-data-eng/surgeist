use super::*;
use crate::grid::subgrid_work::{SubgridWork, take_subgrid_work};
use crate::{
    AlignContent, AlignItems, ComputedOverflow, GridAxisKind, GridLanesFlow, GridPlacement,
    MinSizeOf, Overflow, PreferredSizeOf, ScrollbarWidthOf, SizeContainment, SubgridTrack,
    TrackComponentOf,
};

fn subgrid<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))]
}

fn fixed_tracks<S: LayoutScalar>(sizes: [f64; 2]) -> Vec<TrackComponentOf<S>> {
    sizes
        .map(|size| TrackComponentOf::px(S::from_f64(size)))
        .into()
}

fn px_size<S: LayoutScalar>(width: f64, height: f64) -> Size<PreferredSizeOf<S>> {
    Size::new(width, height).map(|value| PreferredSizeOf::px(S::from_f64(value)))
}

/// Where each chained subgrid sits in its parent's explicit tracks.
#[derive(Clone, Copy)]
enum Span {
    /// Both tracks of both axes; the subgrid covers the parent's whole area.
    Whole,
    /// The second column track and the first row track only.
    SecondColumnFirstRow,
}

impl Span {
    fn placement(self) -> (GridPlacement, GridPlacement) {
        match self {
            Self::Whole => (
                GridPlacement::try_lines(1, 3).unwrap(),
                GridPlacement::try_lines(1, 3).unwrap(),
            ),
            Self::SecondColumnFirstRow => (
                GridPlacement::try_lines(2, 3).unwrap(),
                GridPlacement::try_lines(1, 2).unwrap(),
            ),
        }
    }
}

// Grid chain. Node 0 is a 70x70 Grid with fixed 30px and 40px columns and
// fixed 20px and 50px rows. Nodes 1..=depth are Grid subgrids in both axes,
// each the only item of its parent. The last subgrid holds two stretched
// leaves, `depth + 1` in the first inherited track of each axis and
// `depth + 2` in the last. Every track size is explicit, so no automatic
// track or intrinsic contribution is involved.
fn subgrid_grid_chain<S: LayoutScalar>(depth: u32, span: Span) -> PublicLayoutTreeOf<S> {
    let (column, row) = span.placement();
    let (leaf_column, leaf_row) = match span {
        Span::Whole => (
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(2).unwrap(),
            ],
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(2).unwrap(),
            ],
        ),
        // One inherited track per axis, so both leaves share it.
        Span::SecondColumnFirstRow => (
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(1).unwrap(),
            ],
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(1).unwrap(),
            ],
        ),
    };
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: px_size(70.0, 70.0),
            grid_template_columns: fixed_tracks([30.0, 40.0]),
            grid_template_rows: fixed_tracks([20.0, 50.0]),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        // Only the outermost subgrid is placed by the explicit span. Its
        // descendants fill the inherited tracks of the level above, which is
        // the whole of them for `Whole` and the single track otherwise.
        let (grid_column, grid_row) = if node == 1 {
            (column, row)
        } else {
            match span {
                Span::Whole => (column, row),
                Span::SecondColumnFirstRow => (
                    GridPlacement::try_lines(1, 2).unwrap(),
                    GridPlacement::try_lines(1, 2).unwrap(),
                ),
            }
        };
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Grid,
                grid_column,
                grid_row,
                grid_template_columns: subgrid(),
                grid_template_rows: subgrid(),
                ..NodeInputOf::default()
            },
        );
        tree = tree.children(node - 1, [node]);
    }
    for (index, leaf) in [depth + 1, depth + 2].into_iter().enumerate() {
        tree = tree.style(
            leaf,
            NodeInputOf {
                grid_column: leaf_column[index],
                grid_row: leaf_row[index],
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2])
}

fn assert_box<S: LayoutScalar>(
    batch: &crate::CompletedLayoutBatchOf<u32, S>,
    node: u32,
    location: (f64, f64),
    size: (f64, f64),
) {
    let (_, layout) = batch
        .node_layouts()
        .find(|(candidate, _)| *candidate == node)
        .unwrap_or_else(|| panic!("node {node} is published"));
    let output = layout.single_box().expect("visible box").final_output();
    assert_eq!(
        (output.location.x, output.location.y),
        (S::from_f64(location.0), S::from_f64(location.1)),
        "node {node} location"
    );
    assert_eq!(
        (output.size.width, output.size.height),
        (S::from_f64(size.0), S::from_f64(size.1)),
        "node {node} size"
    );
}

// CSS Grid Level 2, section 9: a subgrid takes the tracks of its parent
// that its grid area spans, and its own items are placed on those tracks.
// A subgrid with the default stretch alignment fills its grid area, so its
// border box is that area; a stretched item fills the area of its tracks.
fn assert_subgrid_grid_chain<S: LayoutScalar>(depth: u32, span: Span) {
    let tree = subgrid_grid_chain::<S>(depth, span);
    let batch = compute_layout(&tree, 0, standalone()).expect("subgrid chain layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (70.0, 70.0));
    match span {
        Span::Whole => {
            for node in 1..=depth {
                assert_box(&batch, node, (0.0, 0.0), (70.0, 70.0));
            }
            // The leaves take the first and the second inherited column and row.
            assert_box(&batch, depth + 1, (0.0, 0.0), (30.0, 20.0));
            assert_box(&batch, depth + 2, (30.0, 20.0), (40.0, 50.0));
        }
        Span::SecondColumnFirstRow => {
            // The outermost subgrid covers column 2 and row 1: x 30, 40x20.
            assert_box(&batch, 1, (30.0, 0.0), (40.0, 20.0));
            for node in 2..=depth {
                assert_box(&batch, node, (0.0, 0.0), (40.0, 20.0));
            }
            // Both leaves share the single inherited track in each axis.
            assert_box(&batch, depth + 1, (0.0, 0.0), (40.0, 20.0));
            assert_box(&batch, depth + 2, (0.0, 0.0), (40.0, 20.0));
        }
    }
}

#[test]
fn shallow_subgrid_grid_chain_inherits_parent_tracks() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::shallow_subgrid_grid_chain_inherits_parent_tracks",
        || {
            for span in [Span::Whole, Span::SecondColumnFirstRow] {
                assert_subgrid_grid_chain::<f32>(3, span);
                assert_subgrid_grid_chain::<f64>(3, span);
            }
        },
    );
}

#[test]
fn deep_subgrid_grid_chain_inherits_parent_tracks_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::deep_subgrid_grid_chain_inherits_parent_tracks_on_bounded_stack",
        || {
            for span in [Span::Whole, Span::SecondColumnFirstRow] {
                assert_subgrid_grid_chain::<f32>(10, span);
                assert_subgrid_grid_chain::<f64>(10, span);
            }
        },
    );
}

// Grid Lanes chain. Node 0 is a 50x14 Grid Lanes container with fixed 20px
// and 30px lanes in the column axis. Nodes 1..=depth are Grid Lanes
// subgrids in the lane axis, each the only item of its parent, spanning both
// lanes. The last holds three leaves, 20px wide and 5px, 7px and 9px high.
fn subgrid_lanes_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let flow = GridLanesFlow::new(GridAxisKind::Column, false, false);
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::GridLanes,
            grid_lanes_flow: flow,
            size: px_size(50.0, 14.0),
            grid_template_columns: fixed_tracks([20.0, 30.0]),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::GridLanes,
                grid_lanes_flow: flow,
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                size: Size::new(
                    PreferredSizeOf::AUTO,
                    PreferredSizeOf::px(S::from_f64(14.0)),
                ),
                grid_template_columns: subgrid(),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
        tree = tree.children(node - 1, [node]);
    }
    for (index, height) in [5.0, 7.0, 9.0].into_iter().enumerate() {
        let leaf = depth + 1 + index as u32;
        tree = tree.style(
            leaf,
            NodeInputOf {
                size: px_size(20.0, height),
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2, depth + 3])
}

// CSS Grid Level 3, Grid Lanes: each item is placed in the lane with the
// least used extent, the first such lane on ties. The 5px item takes lane 1
// and the 7px item lane 2, both being empty at the time. The 9px item then
// goes below the 5px item in lane 1, whose extent 5 is less than 7. A 20px
// item starts at its lane's start edge; lane 2 starts at 20px.
fn assert_subgrid_lanes_chain<S: LayoutScalar>(depth: u32) {
    let tree = subgrid_lanes_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("subgrid lanes chain layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 4);
    assert_box(&batch, 0, (0.0, 0.0), (50.0, 14.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (50.0, 14.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (20.0, 5.0));
    assert_box(&batch, depth + 2, (20.0, 0.0), (20.0, 7.0));
    assert_box(&batch, depth + 3, (0.0, 5.0), (20.0, 9.0));
}

#[test]
fn shallow_subgrid_lanes_chain_inherits_parent_lanes() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::shallow_subgrid_lanes_chain_inherits_parent_lanes",
        || {
            assert_subgrid_lanes_chain::<f32>(3);
            assert_subgrid_lanes_chain::<f64>(3);
        },
    );
}

// One-axis Grid chain. Node 0 is a 70x40 Grid with fixed 30px and 40px
// columns and one fixed 40px row. Nodes 1..=depth are Grid items that inherit
// only the column axis (`grid-template-columns: subgrid`); their rows are
// their own fixed 20px and 30px tracks. Each has `min-height: min-content`
// and `align-self: start`. Every level spans both
// inherited columns and its parent's first row. The last level holds two
// stretched leaves, `depth + 1` in the first column and row and `depth + 2`
// in the second. The non-inherited row axis makes each level a standalone
// Grid for its minimum.
fn standalone_minimum_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: px_size(70.0, 40.0),
            grid_template_columns: fixed_tracks([30.0, 40.0]),
            grid_template_rows: vec![TrackComponentOf::px(S::from_f64(40.0))],
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                    align_self: Some(AlignItems::Start),
                    min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::MIN_CONTENT),
                    grid_template_columns: subgrid(),
                    grid_template_rows: fixed_tracks([20.0, 30.0]),
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    for (leaf, line) in [(depth + 1, 1), (depth + 2, 2)] {
        tree = tree.style(
            leaf,
            NodeInputOf {
                grid_column: GridPlacement::try_line(line).unwrap(),
                grid_row: GridPlacement::try_line(line).unwrap(),
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2])
}

// CSS Grid Level 2, section 9: a subgrid takes the tracks of its parent that
// it spans in each inherited axis, here both columns, and sizes the other axis
// with its own tracks. An auto-height Grid container with `align-self: start`
// is as high as its tracks, 20px plus 30px, and the min-content block size of
// a Grid is that same 50px (CSS Sizing 3, section 5.1), so every level is
// 50px high. Its width is the 70px inherited span, stretched.
fn assert_standalone_minimum_chain<S: LayoutScalar>(depth: u32) {
    let tree = standalone_minimum_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("one-axis subgrid chain layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (70.0, 40.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (70.0, 50.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (30.0, 20.0));
    assert_box(&batch, depth + 2, (30.0, 20.0), (40.0, 30.0));
}

#[test]
fn shallow_one_axis_subgrid_chain_clamps_to_standalone_minimum() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::shallow_one_axis_subgrid_chain_clamps_to_standalone_minimum",
        || {
            assert_standalone_minimum_chain::<f32>(3);
            assert_standalone_minimum_chain::<f64>(3);
        },
    );
}

// CSS Grid Level 2, section 9, with CSS Contain 2 size containment. The item
// inherits both columns from its 100px and 100px parent tracks and has its own
// fixed 30px and 30px rows; size containment sizes it as if it had no content,
// which keeps its tracks. Its width is the 200px of the inherited columns and
// its height the 60px of its rows. `min-height: min-content` on a subgrid that
// inherits only the column axis makes the item a standalone Grid for its
// minimum, whose probe runs size containment before any pass. Its content, a
// 20px box, lies in the first inherited column and row.
fn probed_contained_item<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: px_size(200.0, 200.0),
                grid_template_columns: fixed_tracks([100.0, 100.0]),
                grid_template_rows: fixed_tracks([100.0, 100.0]),
                ..NodeInputOf::default()
            },
        )
        .children(0, [1])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_span(2).unwrap(),
                grid_row: GridPlacement::try_span(2).unwrap(),
                grid_template_columns: subgrid(),
                grid_template_rows: fixed_tracks([30.0, 30.0]),
                size_containment: SizeContainment::BothAxes,
                overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(15.0)).unwrap(),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::MIN_CONTENT),
                ..NodeInputOf::default()
            },
        )
        .children(1, [2])
        .style(
            2,
            NodeInputOf {
                size: px_size(20.0, 20.0),
                ..NodeInputOf::default()
            },
        )
}

fn assert_probed_contained_item<S: LayoutScalar>() {
    let tree = probed_contained_item::<S>();
    let batch = compute_layout(&tree, 0, standalone()).expect("contained subgrid layout");
    assert_eq!(batch.node_layouts().len(), 3);
    assert_box(&batch, 1, (0.0, 0.0), (200.0, 60.0));
    assert_box(&batch, 2, (0.0, 0.0), (20.0, 20.0));
}

#[test]
fn size_contained_subgrid_item_keeps_its_tracks_when_its_minimum_is_probed() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::size_contained_subgrid_item_keeps_its_tracks_when_its_minimum_is_probed",
        || {
            assert_probed_contained_item::<f32>();
            assert_probed_contained_item::<f64>();
        },
    );
}

fn nan_height<S: LayoutScalar>(_: crate::LeafMeasureInputOf<S>) -> Size<S> {
    Size::new(S::from_f64(10.0), S::NAN)
}

// A leaf at the end of a subgrid chain returns a non-finite height. The
// measurement protocol refuses it at the leaf, and the refusal reaches the
// caller unchanged through every contextual Grid request above it.
fn assert_chain_failure_surfaces_unchanged<S: LayoutScalar>() {
    let depth = 3;
    let leaf = depth + 1;
    let tree = subgrid_grid_chain::<S>(depth, Span::Whole).measure_with(leaf, nan_height::<S>);
    let error = compute_layout(&tree, 0, standalone()).expect_err("non-finite leaf height");
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(leaf));
    assert_eq!(error.operation(), crate::LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::MeasurementOutput(output))
            if output.axis() == crate::PhysicalAxis::Vertical
                && matches!(output.error(), crate::NonNegativeFiniteScalarErrorOf::NonFinite { value } if value.to_f64().is_nan())
    ));
}

#[test]
fn subgrid_chain_leaf_refusal_surfaces_unchanged() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::subgrid_chain_leaf_refusal_surfaces_unchanged",
        || {
            assert_chain_failure_surfaces_unchanged::<f32>();
            assert_chain_failure_surfaces_unchanged::<f64>();
        },
    );
}

// Work growth of a chain of nested subgrids. A level needs one contextual Grid
// computation of its subgrid child for placement, which may probe the child's
// standalone minimum once and refresh its baselines once, so those grow with
// the depth by at most one level's share. Intrinsic traversal prepares each
// wrapper once for each distinct combination of axis, sizing stage and parent
// tracks that the root asks for, and a Grid Lanes level adds the contextual
// computations of its lane candidates. Each is a fixed number per level: the
// counts below bound the growth from one depth to the next. None of them may
// grow with the number of paths through the chain, which would make the growth
// itself grow with the depth.
const WRAPPER_PREPARATIONS_PER_LEVEL: usize = 24;
const CONTEXTUAL_GRIDS_PER_LEVEL: usize = 14;
const BASELINE_REFRESHES_PER_LEVEL: usize = 1;
const MINIMUM_PROBES_PER_LEVEL: usize = 2;
const WORK_DEPTHS: [u32; 4] = [2, 3, 4, 5];

fn assert_subgrid_work_grows_linearly(name: &str, work: &[SubgridWork]) {
    for (depth, pair) in WORK_DEPTHS.iter().zip(work.windows(2)) {
        let (lower, upper) = (pair[0], pair[1]);
        for (what, lower, upper, limit) in [
            (
                "wrapper preparations",
                lower.wrapper_preparations,
                upper.wrapper_preparations,
                WRAPPER_PREPARATIONS_PER_LEVEL,
            ),
            (
                "contextual Grid computations",
                lower.contextual_grids,
                upper.contextual_grids,
                CONTEXTUAL_GRIDS_PER_LEVEL,
            ),
            (
                "baseline refreshes",
                lower.baseline_refreshes,
                upper.baseline_refreshes,
                BASELINE_REFRESHES_PER_LEVEL,
            ),
            (
                "standalone-minimum probes",
                lower.minimum_probes,
                upper.minimum_probes,
                MINIMUM_PROBES_PER_LEVEL,
            ),
        ] {
            assert!(
                upper <= lower + limit,
                "{name}: {what} grow from {lower} at depth {depth} to {upper} at depth {}, \
                 more than {limit} for one level",
                depth + 1,
            );
        }
    }
}

fn subgrid_work_of<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> SubgridWork {
    take_subgrid_work();
    compute_layout(tree, 0, standalone()).expect("subgrid chain layout");
    take_subgrid_work()
}

/// The work of the chain at each of `WORK_DEPTHS`. The thread has a stack that
/// holds the native recursion of the deepest chain.
fn subgrid_work_by_depth<S: LayoutScalar>(
    chain: impl Fn(u32) -> PublicLayoutTreeOf<S> + Send + 'static,
) -> Vec<SubgridWork> {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            WORK_DEPTHS
                .iter()
                .map(|&depth| subgrid_work_of(&chain(depth)))
                .collect()
        })
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn subgrid_grid_chain_work_grows_linearly_with_depth() {
    for span in [Span::Whole, Span::SecondColumnFirstRow] {
        assert_subgrid_work_grows_linearly(
            "Grid chain",
            &subgrid_work_by_depth(move |depth| subgrid_grid_chain::<f32>(depth, span)),
        );
        assert_subgrid_work_grows_linearly(
            "Grid chain",
            &subgrid_work_by_depth(move |depth| subgrid_grid_chain::<f64>(depth, span)),
        );
    }
}

#[test]
fn subgrid_lanes_chain_work_grows_linearly_with_depth() {
    assert_subgrid_work_grows_linearly(
        "Grid Lanes chain",
        &subgrid_work_by_depth(subgrid_lanes_chain::<f32>),
    );
    assert_subgrid_work_grows_linearly(
        "Grid Lanes chain",
        &subgrid_work_by_depth(subgrid_lanes_chain::<f64>),
    );
}

#[test]
fn one_axis_subgrid_chain_work_grows_linearly_with_depth() {
    assert_subgrid_work_grows_linearly(
        "one-axis chain",
        &subgrid_work_by_depth(standalone_minimum_chain::<f32>),
    );
    assert_subgrid_work_grows_linearly(
        "one-axis chain",
        &subgrid_work_by_depth(standalone_minimum_chain::<f64>),
    );
}

// Intrinsic Grid chain. Node 0 is a 70x70 Grid with two automatic columns and
// two automatic rows. Nodes 1..=depth are Grid subgrids in both axes, each
// the only item of its parent and spanning both tracks of each axis. The last
// subgrid holds two leaves of definite size: `depth + 1` is 30x20 in the first
// track of each axis and `depth + 2` is 40x50 in the second. The automatic
// tracks take their sizes from those leaves through every subgrid level.
fn intrinsic_subgrid_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    intrinsic_subgrid_chain_from(0, depth)
}

// The same chain with its root numbered `first`, so that the chain can sit
// below another node.
fn intrinsic_subgrid_chain_from<S: LayoutScalar>(first: u32, depth: u32) -> PublicLayoutTreeOf<S> {
    let auto_tracks = || vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO];
    let last = first + depth;
    let mut tree = PublicLayoutTreeOf::new().style(
        first,
        NodeInputOf {
            display: Display::Grid,
            size: px_size(70.0, 70.0),
            grid_template_columns: auto_tracks(),
            grid_template_rows: auto_tracks(),
            ..NodeInputOf::default()
        },
    );
    for node in first + 1..=last {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_row: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: subgrid(),
                    grid_template_rows: subgrid(),
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    for (leaf, line, width, height) in [(last + 1, 1, 30.0, 20.0), (last + 2, 2, 40.0, 50.0)] {
        tree = tree.style(
            leaf,
            NodeInputOf {
                grid_column: GridPlacement::try_line(line).unwrap(),
                grid_row: GridPlacement::try_line(line).unwrap(),
                size: px_size(width, height),
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(last, [last + 1, last + 2])
}

// CSS Grid Level 2, section 9: the items of a subgrid take part in the track
// sizing of the parent grid as if they were its own items. The automatic
// tracks therefore size to the leaves, 30px and 40px wide, 20px and 50px high,
// and the root's 70px by 70px box has no free space to distribute. Every
// subgrid fills its grid area, which is the whole root.
fn assert_intrinsic_subgrid_chain<S: LayoutScalar>(depth: u32) {
    let tree = intrinsic_subgrid_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("intrinsic subgrid chain layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (70.0, 70.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (70.0, 70.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (30.0, 20.0));
    assert_box(&batch, depth + 2, (30.0, 20.0), (40.0, 50.0));
}

#[test]
fn shallow_intrinsic_subgrid_chain_sizes_tracks_from_its_leaves() {
    assert_intrinsic_subgrid_chain::<f32>(3);
    assert_intrinsic_subgrid_chain::<f64>(3);
}

#[test]
fn intrinsic_subgrid_chain_work_grows_linearly_with_depth() {
    // Intrinsic contributions reach the root through every level in a bounded
    // number of traversals of the chain.
    assert_subgrid_work_grows_linearly(
        "intrinsic chain",
        &subgrid_work_by_depth(intrinsic_subgrid_chain::<f32>),
    );
    assert_subgrid_work_grows_linearly(
        "intrinsic chain",
        &subgrid_work_by_depth(intrinsic_subgrid_chain::<f64>),
    );
}

// Block fragmentation stages block bodies, scope receipts and margin groups that
// the staged effects do not cover, so a fragmented layout reuses no subgrid
// computation and lays out as it does with every computation repeated. Only
// Block boxes fragment, so the chain sits inside a Block that size containment
// makes atomic, below a Block root.
#[test]
fn fragmented_subgrid_chain_reuses_nothing_and_lays_out_the_same() {
    let nodes = 7;
    let tree = intrinsic_subgrid_chain_from::<f64>(2, nodes - 5)
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: px_size(70.0, 70.0),
                size_containment: crate::SizeContainment::BothAxes,
                ..NodeInputOf::default()
            },
        )
        .children(0, [1])
        .children(1, [2]);
    let layout = || {
        take_subgrid_work();
        let batch = super::scoped_flex::scoped_layout(&tree, nodes);
        (format!("{batch:?}"), take_subgrid_work())
    };
    let (reused, reused_work) = layout();
    let (repeated, repeated_work) = crate::grid::subgrid_work::with_subgrid_reuse_disabled(layout);
    assert!(
        reused == repeated,
        "the fragmented layout is the same with reuse"
    );
    assert!(
        reused_work.wrapper_preparations > 0,
        "the fragmented layout prepares subgrid wrappers: {reused_work:?}"
    );
    assert_eq!(
        reused_work.wrapper_preparations, repeated_work.wrapper_preparations,
        "a fragmented session reuses no wrapper preparation"
    );
}

/// The staged writes replayed by chains of `depths`. Each reuse replays the
/// writes of its subtree, as few as give the same result, so doubling the depth
/// at most quadruples them. A log that kept every repeat of a subtree would
/// multiply them by a power of two for each level.
fn effects_replayed_by_depth<S: LayoutScalar>(
    chain: impl Fn(u32) -> PublicLayoutTreeOf<S> + Send + 'static,
    depths: [u32; 3],
) -> [usize; 3] {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || depths.map(|depth| subgrid_work_of(&chain(depth)).effects_replayed))
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn subgrid_chain_effects_replayed_grow_at_most_quadratically_with_depth() {
    for [shallow, middle, deep] in [
        effects_replayed_by_depth(
            |depth| subgrid_grid_chain::<f64>(depth, Span::Whole),
            [8, 16, 24],
        ),
        effects_replayed_by_depth(intrinsic_subgrid_chain::<f64>, [8, 16, 24]),
        effects_replayed_by_depth(subgrid_lanes_chain::<f64>, [8, 16, 24]),
    ] {
        // Quadratic growth multiplies the count by at most four from depth 8 to 16 and by
        // about two from 16 to 24; an exponential one, however mild, exceeds one of the two.
        assert!(
            middle <= 4 * shallow + 64,
            "{middle} writes replayed at depth 16 against {shallow} at depth 8"
        );
        assert!(
            deep <= 3 * middle + 64,
            "{deep} writes replayed at depth 24 against {middle} at depth 16"
        );
    }
}

// The chain with a `display: none` item in every Grid of it. A hidden item stages its geometry
// during final placement, and a hidden write removes the stores its node staged before it.
fn hidden_item_subgrid_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let mut tree = intrinsic_subgrid_chain::<S>(depth);
    for level in 0..=depth {
        let hidden = depth + 3 + level;
        tree = tree.style(
            hidden,
            NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            },
        );
        let items: Vec<u32> = if level < depth {
            vec![level + 1, hidden]
        } else {
            vec![depth + 1, depth + 2, hidden]
        };
        tree = tree.children(level, items);
    }
    tree
}

// A hidden item does not stop a subgrid computation from being reused, so the chain with a
// hidden item in every Grid prepares as many wrappers as the chain without, and lays out as it
// does with every computation repeated.
#[test]
fn hidden_items_in_a_subgrid_chain_leave_the_reuse_on_and_the_layout_the_same() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(|| {
            let layout = |tree: &PublicLayoutTreeOf<f64>| {
                take_subgrid_work();
                let batch = compute_layout(tree, 0, standalone()).expect("hidden item chain");
                (format!("{batch:?}"), take_subgrid_work())
            };
            let shallow = hidden_item_subgrid_chain::<f64>(3);
            let (reused, _) = layout(&shallow);
            let (repeated, _) =
                crate::grid::subgrid_work::with_subgrid_reuse_disabled(|| layout(&shallow));
            assert!(
                reused == repeated,
                "the hidden items lay out the same with reuse"
            );
            let (_, hidden) = layout(&hidden_item_subgrid_chain::<f64>(10));
            let (_, plain) = layout(&intrinsic_subgrid_chain::<f64>(10));
            assert_eq!(
                hidden.wrapper_preparations, plain.wrapper_preparations,
                "hidden items add no wrapper preparation: {hidden:?} against {plain:?}"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
