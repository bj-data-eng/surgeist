use super::contextual_grid_children::{assert_box, px_size, subgrid};
use super::*;
use crate::{
    grid::GridAxisKind,
    node_input::{
        AlignContent, AlignItems, ComputedOverflow, GridLanesFlow, GridPlacement, Overflow,
        ScrollbarWidthOf,
    },
    value::TrackComponentOf,
};

// Chains whose items size intrinsic tracks of their parent. Every level is an
// ordinary item of the level above, so its intrinsic contribution is a Grid
// computed with its parent's tracks as context. A level asks the level below
// for the same few intrinsic queries however many of its own queries ask, and
// an answered query is not computed again, so the work grows with the depth
// (the work bounds below). The deep depths run on the 2 MiB stack of the
// bounded test, and each finishes in about a second in a debug build. The
// stacking chain is the exception: its lane children ask for contributions,
// which are computed again for each query, so its work grows about four-fold
// per level and its depth stays small.

fn auto_tracks<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO]
}

fn line(number: isize) -> GridPlacement {
    GridPlacement::try_line(number).unwrap()
}

fn lines(start: isize, end: isize) -> GridPlacement {
    GridPlacement::try_lines(start, end).unwrap()
}

/// A definite-size leaf in one cell of the innermost level.
fn leaf<S: LayoutScalar>(column: isize, row: isize, width: f64, height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        grid_column: line(column),
        grid_row: line(row),
        size: px_size(width, height),
        ..NodeInputOf::default()
    }
}

fn layout_of<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    nodes: usize,
) -> crate::output::CompletedLayoutBatchOf<u32, S> {
    let batch = compute_layout(tree, 0, standalone()).expect("chain layout");
    assert_eq!(batch.node_layouts().len(), nodes);
    batch
}

// One-axis Grid chain. Node 0 is a 70px wide Grid with two automatic columns
// and one automatic row. Nodes 1..=depth are Grid items that take both columns
// of their parent as subgrid columns (`grid-template-columns: subgrid`) and
// have two automatic rows of their own, with `align-self: start`. Every level
// is the only item of its parent, in the parent's first row. The last level
// holds two leaves: `depth + 1` is 30x20 in its first column and row and
// `depth + 2` is 40x30 in the second of each.
fn auto_column_chain<S: LayoutScalar>(
    depth: u32,
    root_height: f64,
    scrolled: bool,
) -> PublicLayoutTreeOf<S> {
    let scroll = |style: &mut NodeInputOf<S>| {
        if scrolled {
            style.overflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
            style.scrollbar_width = ScrollbarWidthOf::try_new(S::from_f64(15.0)).unwrap();
        }
    };
    let mut root = NodeInputOf {
        display: Display::Grid,
        size: px_size(70.0, root_height),
        grid_template_columns: auto_tracks(),
        grid_template_rows: vec![TrackComponentOf::AUTO],
        ..NodeInputOf::default()
    };
    scroll(&mut root);
    let mut tree = PublicLayoutTreeOf::new().style(0, root);
    for node in 1..=depth {
        let mut level = NodeInputOf {
            display: Display::Grid,
            grid_column: lines(1, 3),
            grid_row: lines(1, 2),
            align_self: Some(AlignItems::Start),
            grid_template_columns: subgrid(),
            grid_template_rows: auto_tracks(),
            ..NodeInputOf::default()
        };
        scroll(&mut level);
        tree = tree.style(node, level).children(node - 1, [node]);
    }
    tree = tree
        .style(depth + 1, leaf(1, 1, 30.0, 20.0))
        .style(depth + 2, leaf(2, 2, 40.0, 30.0));
    tree.children(depth, [depth + 1, depth + 2])
}

// CSS Grid Level 2, section 9: a subgrid takes the tracks of its parent that
// it spans in the inherited axis, and its items take part in the sizing of
// those tracks as items of the parent. The root's automatic columns therefore
// size to the leaves, 30px and 40px, which fill its 70px width. An item that
// spans both columns is 70px wide. Its own two automatic rows size to its
// content: the nested level, 50px high, in the first row and nothing in the
// second, and at the innermost level the leaves, 20px and 30px, so every level
// is 50px high with `align-self: start`. The nested level starts at its
// parent's content box origin. The leaves take the first and the second
// column, and the first and the second row of the innermost level.
fn assert_auto_column_chain<S: LayoutScalar>(depth: u32, root_height: f64, scrolled: bool) {
    let tree = auto_column_chain::<S>(depth, root_height, scrolled);
    let batch = layout_of(&tree, depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (70.0, root_height));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (70.0, 50.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (30.0, 20.0));
    assert_box(&batch, depth + 2, (30.0, 20.0), (40.0, 30.0));
}

const AUTO_COLUMN_DEPTH_F32: u32 = 24;
const AUTO_COLUMN_DEPTH_F64: u32 = 24;

#[test]
fn shallow_auto_column_subgrid_chain_sizes_columns_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::shallow_auto_column_subgrid_chain_sizes_columns_from_leaves",
        || {
            assert_auto_column_chain::<f32>(3, 40.0, false);
            assert_auto_column_chain::<f64>(3, 40.0, false);
        },
    );
}

#[test]
fn deep_auto_column_subgrid_chain_sizes_columns_from_leaves_f32() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_auto_column_subgrid_chain_sizes_columns_from_leaves_f32",
        || assert_auto_column_chain::<f32>(AUTO_COLUMN_DEPTH_F32, 40.0, false),
    );
}

#[test]
fn deep_auto_column_subgrid_chain_sizes_columns_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_auto_column_subgrid_chain_sizes_columns_from_leaves_f64",
        || assert_auto_column_chain::<f64>(AUTO_COLUMN_DEPTH_F64, 40.0, false),
    );
}

// The same chain with `overflow: auto` and a 15px scrollbar width on every
// container. CSS Overflow 3: a scrollbar appears only for overflowing content.
// The root is 60px high, which holds its 50px row, and every level holds its
// content, so there is no overflow, no scrollbar and no reserved gutter, and
// the geometry is that of the chain without scrolling.
const SCROLLED_COLUMN_DEPTH: u32 = 8;

#[test]
fn shallow_scrolled_auto_column_subgrid_chain_sizes_columns_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::shallow_scrolled_auto_column_subgrid_chain_sizes_columns_from_leaves",
        || {
            assert_auto_column_chain::<f32>(2, 60.0, true);
            assert_auto_column_chain::<f64>(2, 60.0, true);
        },
    );
}

#[test]
fn deep_scrolled_auto_column_subgrid_chain_sizes_columns_from_leaves_f32() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_scrolled_auto_column_subgrid_chain_sizes_columns_from_leaves_f32",
        || assert_auto_column_chain::<f32>(SCROLLED_COLUMN_DEPTH, 60.0, true),
    );
}

#[test]
fn deep_scrolled_auto_column_subgrid_chain_sizes_columns_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_scrolled_auto_column_subgrid_chain_sizes_columns_from_leaves_f64",
        || assert_auto_column_chain::<f64>(SCROLLED_COLUMN_DEPTH, 60.0, true),
    );
}

// Alternating chain. Node 0 is a 70x70 Grid with two automatic columns and two
// automatic rows. Odd levels are subgrids in both axes and even levels are
// subgrids in the columns only, with two automatic rows of their own. Every
// level is the only item of its parent and spans both of the parent's columns
// and rows. The last level, `depth` being even, holds two leaves: `depth + 1`
// is 30x20 in the first column and row and `depth + 2` is 40x50 in the second.
fn alternating_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: px_size(70.0, 70.0),
            grid_template_columns: auto_tracks(),
            grid_template_rows: auto_tracks(),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        let both_axes = node % 2 == 1;
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: lines(1, 3),
                    grid_row: lines(1, 3),
                    grid_template_columns: subgrid(),
                    grid_template_rows: if both_axes { subgrid() } else { auto_tracks() },
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    tree = tree
        .style(depth + 1, leaf(1, 1, 30.0, 20.0))
        .style(depth + 2, leaf(2, 2, 40.0, 50.0));
    tree.children(depth, [depth + 1, depth + 2])
}

// CSS Grid Level 2, section 9. The columns of every level are the root's, which
// size to the leaves, 30px and 40px, and fill the 70px width. The innermost
// level's own rows size to its leaves, 20px and 50px, so it is 70px high. Each
// level above holds the next in all of its rows, which have no other content
// and so share the 70px; the root's 70px height holds them. Every level
// stretches over the whole 70x70 area of its parent's tracks. The leaves take
// the first and the second column and row of the innermost level.
fn assert_alternating_chain<S: LayoutScalar>(depth: u32) {
    let tree = alternating_chain::<S>(depth);
    let batch = layout_of(&tree, depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (70.0, 70.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (70.0, 70.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (30.0, 20.0));
    assert_box(&batch, depth + 2, (30.0, 20.0), (40.0, 50.0));
}

const ALTERNATING_DEPTH: u32 = 32;

#[test]
fn shallow_alternating_subgrid_chain_sizes_tracks_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::shallow_alternating_subgrid_chain_sizes_tracks_from_leaves",
        || {
            assert_alternating_chain::<f32>(4);
            assert_alternating_chain::<f64>(4);
        },
    );
}

#[test]
fn deep_alternating_subgrid_chain_sizes_tracks_from_leaves_f32() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_alternating_subgrid_chain_sizes_tracks_from_leaves_f32",
        || assert_alternating_chain::<f32>(ALTERNATING_DEPTH),
    );
}

#[test]
fn deep_alternating_subgrid_chain_sizes_tracks_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_alternating_subgrid_chain_sizes_tracks_from_leaves_f64",
        || assert_alternating_chain::<f64>(ALTERNATING_DEPTH),
    );
}

// Grid Lanes chain with automatic lanes. Node 0 is a 40x14 Grid Lanes
// container with two automatic lanes in the column axis. Nodes 1..=depth are
// Grid Lanes subgrids in the lane axis, each the only item of its parent and
// spanning both lanes. The last level holds three leaves, 20px wide and 5px,
// 7px and 9px high.
fn auto_lane_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let flow = GridLanesFlow::new(GridAxisKind::Column, false, false);
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::GridLanes,
            grid_lanes_flow: flow,
            size: px_size(40.0, 14.0),
            grid_template_columns: auto_tracks(),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: flow,
                    grid_column: lines(1, 3),
                    grid_template_columns: subgrid(),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    for (index, height) in [5.0, 7.0, 9.0].into_iter().enumerate() {
        tree = tree.style(
            depth + 1 + index as u32,
            NodeInputOf {
                size: px_size(20.0, height),
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2, depth + 3])
}

// CSS Grid Level 3, Grid Lanes: each item is placed in the lane with the
// least used extent, the first such lane on ties. The 5px item takes lane 1,
// the 7px item lane 2, and the 9px item goes below the 5px item in lane 1,
// whose extent 5 is less than 7. The items of a subgrid take part in the
// sizing of the parent's automatic lanes, so each lane is 20px wide, which
// fills the 40px width. Every subgrid spans both lanes, is 40px wide, and its
// automatic block size is that of its longest lane, 5 + 9 = 14px.
fn assert_auto_lane_chain<S: LayoutScalar>(depth: u32) {
    let tree = auto_lane_chain::<S>(depth);
    let batch = layout_of(&tree, depth as usize + 4);
    assert_box(&batch, 0, (0.0, 0.0), (40.0, 14.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (40.0, 14.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (20.0, 5.0));
    assert_box(&batch, depth + 2, (20.0, 0.0), (20.0, 7.0));
    assert_box(&batch, depth + 3, (0.0, 5.0), (20.0, 9.0));
}

const AUTO_LANE_DEPTH: u32 = 24;

#[test]
fn shallow_auto_lane_subgrid_chain_sizes_lanes_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::shallow_auto_lane_subgrid_chain_sizes_lanes_from_leaves",
        || {
            assert_auto_lane_chain::<f32>(3);
            assert_auto_lane_chain::<f64>(3);
        },
    );
}

#[test]
fn deep_auto_lane_subgrid_chain_sizes_lanes_from_leaves_f32() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_auto_lane_subgrid_chain_sizes_lanes_from_leaves_f32",
        || assert_auto_lane_chain::<f32>(AUTO_LANE_DEPTH),
    );
}

#[test]
fn deep_auto_lane_subgrid_chain_sizes_lanes_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_auto_lane_subgrid_chain_sizes_lanes_from_leaves_f64",
        || assert_auto_lane_chain::<f64>(AUTO_LANE_DEPTH),
    );
}

// Grid Lanes chain of Grid items with a subgrid stacking axis. Node 0 is a
// 20x21 Grid Lanes container with one automatic lane in the column axis.
// Nodes 1..=depth are Grid containers with `grid-template-rows: subgrid`, each
// the only item of its parent, so a lane child that requires a context at every
// level. The last level holds three leaves, 20px wide and 5px, 7px and 9px high.
fn stacking_subgrid_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::GridLanes,
            grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
            size: px_size(20.0, 21.0),
            grid_template_columns: vec![TrackComponentOf::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::Grid,
                    grid_template_rows: subgrid(),
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    for (index, height) in [5.0, 7.0, 9.0].into_iter().enumerate() {
        tree = tree.style(
            depth + 1 + index as u32,
            NodeInputOf {
                size: px_size(20.0, height),
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2, depth + 3])
}

// CSS Grid Level 3, Grid Lanes. The one automatic lane takes the 20px width of
// the leaves, which is the container's width. Every level fills the lane and
// holds the leaves of the innermost level stacked in it in source order, 5px,
// 7px and 9px high, so it is 5 + 7 + 9 = 21px high.
fn assert_stacking_subgrid_chain<S: LayoutScalar>(depth: u32) {
    let tree = stacking_subgrid_chain::<S>(depth);
    let batch = layout_of(&tree, depth as usize + 4);
    assert_box(&batch, 0, (0.0, 0.0), (20.0, 21.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (20.0, 21.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (20.0, 5.0));
    assert_box(&batch, depth + 2, (0.0, 5.0), (20.0, 7.0));
    assert_box(&batch, depth + 3, (0.0, 12.0), (20.0, 9.0));
}

// The work of this chain grows about four-fold per level, so its depth is
// small.
const STACKING_DEPTH: u32 = 6;

#[test]
fn shallow_stacking_subgrid_chain_in_a_lane_sizes_the_lane_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::shallow_stacking_subgrid_chain_in_a_lane_sizes_the_lane_from_leaves",
        || {
            assert_stacking_subgrid_chain::<f32>(3);
            assert_stacking_subgrid_chain::<f64>(3);
        },
    );
}

#[test]
fn deep_stacking_subgrid_chain_in_a_lane_sizes_the_lane_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::deep_stacking_subgrid_chain_in_a_lane_sizes_the_lane_from_leaves_f64",
        || assert_stacking_subgrid_chain::<f64>(STACKING_DEPTH),
    );
}

// A leaf whose measurement fails. The refusal of the host reaches the caller
// unchanged from the leaf, through every contextual Grid request above it, and
// every effect capture the requests held is closed: a session dropped with a
// capture open fails its test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LeafFailure;

struct FailingLeaf<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    leaf: u32,
}

impl<S: LayoutScalar> crate::tree::Traverse for FailingLeaf<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as crate::tree::Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        crate::tree::Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> crate::tree::LayoutTree for FailingLeaf<S> {
    type MeasureError = LeafFailure;

    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: u32) -> crate::node_input::LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: u32) -> bool {
        node == self.leaf
    }

    fn measure_leaf(
        &self,
        node: u32,
        _input: crate::measurement::LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, LeafFailure>> {
        (node == self.leaf).then_some(Err(LeafFailure))
    }
}

/// `tree` with `leaves` measured by the host: they have no size of their own,
/// so that their measurement is asked for. A leaf of a Grid sits in `cell`.
fn measured_leaves<S: LayoutScalar>(
    mut tree: PublicLayoutTreeOf<S>,
    leaves: &[(u32, Option<(isize, isize)>)],
) -> PublicLayoutTreeOf<S> {
    for &(node, cell) in leaves {
        let style = match cell {
            Some((column, row)) => NodeInputOf {
                grid_column: line(column),
                grid_row: line(row),
                ..NodeInputOf::default()
            },
            None => NodeInputOf::default(),
        };
        tree = tree.style(node, style);
    }
    tree
}

fn assert_leaf_failure_surfaces_unchanged<S: LayoutScalar>(tree: PublicLayoutTreeOf<S>, leaf: u32) {
    let document = FailingLeaf { tree, leaf };
    let error =
        compute_layout(&document, 0, standalone()).expect_err("the leaf cannot be measured");
    assert_eq!(error.site(), crate::error::LayoutErrorSiteOf::Node(leaf));
    assert_eq!(
        error.operation(),
        crate::error::LayoutOperation::LeafMeasurement
    );
    assert!(
        matches!(
            error.kind(),
            crate::error::LayoutErrorKindOf::Measurement(LeafFailure)
        ),
        "{error:?}"
    );
}

fn assert_auto_column_leaf_failure<S: LayoutScalar>(depth: u32, scrolled: bool) {
    let tree = measured_leaves(
        auto_column_chain::<S>(depth, 60.0, scrolled),
        &[(depth + 1, Some((1, 1))), (depth + 2, Some((2, 2)))],
    );
    assert_leaf_failure_surfaces_unchanged(tree.clone(), depth + 1);
    assert_leaf_failure_surfaces_unchanged(tree, depth + 2);
}

fn assert_alternating_leaf_failure<S: LayoutScalar>(depth: u32) {
    let tree = measured_leaves(
        alternating_chain::<S>(depth),
        &[(depth + 1, Some((1, 1))), (depth + 2, Some((2, 2)))],
    );
    assert_leaf_failure_surfaces_unchanged(tree.clone(), depth + 1);
    assert_leaf_failure_surfaces_unchanged(tree, depth + 2);
}

fn assert_lane_leaf_failure<S: LayoutScalar>(
    tree: impl Fn(u32) -> PublicLayoutTreeOf<S>,
    depth: u32,
) {
    let tree = measured_leaves(
        tree(depth),
        &[(depth + 1, None), (depth + 2, None), (depth + 3, None)],
    );
    for leaf in [depth + 1, depth + 2, depth + 3] {
        assert_leaf_failure_surfaces_unchanged(tree.clone(), leaf);
    }
}

// The same refusals with the reuse of subgrid computations and with every
// computation repeated: a probe that fails is recorded nowhere, so a later
// request of it raises the same refusal, and the error is the one the repeated
// layout raises.
fn layout_refusal<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>, leaf: u32) -> String {
    let document = FailingLeaf {
        tree: tree.clone(),
        leaf,
    };
    let error =
        compute_layout(&document, 0, standalone()).expect_err("the leaf cannot be measured");
    assert_eq!(error.site(), crate::error::LayoutErrorSiteOf::Node(leaf));
    assert_eq!(
        error.operation(),
        crate::error::LayoutOperation::LeafMeasurement
    );
    assert!(
        matches!(
            error.kind(),
            crate::error::LayoutErrorKindOf::Measurement(LeafFailure)
        ),
        "{error:?}"
    );
    format!("{error:?}")
}

fn assert_refusal_is_the_same_without_reuse<S: LayoutScalar>(
    tree: PublicLayoutTreeOf<S>,
    leaves: &[u32],
) {
    for &leaf in leaves {
        let reused = layout_refusal(&tree, leaf);
        let repeated =
            crate::grid::subgrid_work::with_subgrid_reuse_disabled(|| layout_refusal(&tree, leaf));
        assert_eq!(reused, repeated, "leaf {leaf}");
    }
}

#[test]
fn auto_column_chain_leaf_refusal_is_the_same_with_and_without_reuse() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::auto_column_chain_leaf_refusal_is_the_same_with_and_without_reuse",
        || {
            let leaves = [(4, Some((1, 1))), (5, Some((2, 2)))];
            assert_refusal_is_the_same_without_reuse(
                measured_leaves(auto_column_chain::<f32>(3, 60.0, false), &leaves),
                &[4, 5],
            );
            assert_refusal_is_the_same_without_reuse(
                measured_leaves(auto_column_chain::<f64>(3, 60.0, false), &leaves),
                &[4, 5],
            );
        },
    );
}

#[test]
fn auto_lane_chain_leaf_refusal_is_the_same_with_and_without_reuse() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::auto_lane_chain_leaf_refusal_is_the_same_with_and_without_reuse",
        || {
            let leaves = [(4, None), (5, None), (6, None)];
            assert_refusal_is_the_same_without_reuse(
                measured_leaves(auto_lane_chain::<f32>(3), &leaves),
                &[4, 5, 6],
            );
            assert_refusal_is_the_same_without_reuse(
                measured_leaves(auto_lane_chain::<f64>(3), &leaves),
                &[4, 5, 6],
            );
        },
    );
}

#[test]
fn auto_column_subgrid_chain_leaf_refusal_surfaces_unchanged() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::auto_column_subgrid_chain_leaf_refusal_surfaces_unchanged",
        || {
            assert_auto_column_leaf_failure::<f32>(3, false);
            assert_auto_column_leaf_failure::<f64>(3, false);
        },
    );
}

#[test]
fn scrolled_auto_column_subgrid_chain_leaf_refusal_surfaces_unchanged() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::scrolled_auto_column_subgrid_chain_leaf_refusal_surfaces_unchanged",
        || {
            assert_auto_column_leaf_failure::<f32>(2, true);
            assert_auto_column_leaf_failure::<f64>(2, true);
        },
    );
}

#[test]
fn alternating_subgrid_chain_leaf_refusal_surfaces_unchanged() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::alternating_subgrid_chain_leaf_refusal_surfaces_unchanged",
        || {
            assert_alternating_leaf_failure::<f32>(4);
            assert_alternating_leaf_failure::<f64>(4);
        },
    );
}

#[test]
fn auto_lane_subgrid_chain_leaf_refusal_surfaces_unchanged() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::auto_lane_subgrid_chain_leaf_refusal_surfaces_unchanged",
        || {
            assert_lane_leaf_failure::<f32>(auto_lane_chain::<f32>, 3);
            assert_lane_leaf_failure::<f64>(auto_lane_chain::<f64>, 3);
        },
    );
}

#[test]
fn stacking_subgrid_chain_leaf_refusal_surfaces_unchanged() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::stacking_subgrid_chain_leaf_refusal_surfaces_unchanged",
        || {
            assert_lane_leaf_failure::<f32>(stacking_subgrid_chain::<f32>, 3);
            assert_lane_leaf_failure::<f64>(stacking_subgrid_chain::<f64>, 3);
        },
    );
}

// Work bounds. Laying out a chain of nested subgrids costs time roughly
// proportional to its depth (issue #2883), so the contextual Grid
// computations of a chain may grow from one depth to the next by the number of
// distinct requests one level makes, not by a multiple of what the level
// above made.
//
// One level whose own rows are sized from its content asks its single item,
// the next level, for each of three contributions of the intrinsic track
// sizing algorithm (CSS Grid Level 1, section 11.5: the item's minimum
// contribution, its min-content contribution and its max-content
// contribution) in each of the two sizing stages that size the second axis
// (the initial and the revisit). A parent that is itself a subgrid describes
// its tracks, named lines and minimum functions differently from a parent
// that declares them, so the same query can be made under two descriptions of
// its parent. The level is also laid out once for its final geometry and
// refreshed once for its baselines. A request is computed once for its node,
// input, stage and parent facts however many parents ask for it, so one level
// adds at most
const OWN_ROW_REQUESTS_PER_LEVEL: usize = 2 * 3 * 2 + 1 + 1;
// contextual computations. A Grid Lanes level is also a lane-packing
// candidate of its parent, which asks for the same contributions again.
const LANE_REQUESTS_PER_LEVEL: usize = 2 * OWN_ROW_REQUESTS_PER_LEVEL;
// A scroll container level observes its scrollbar gutter with a perform-layout
// probe, run again after the scrollbars settle (at most two passes), and each
// pass lays out every level below it. The increase from one depth to the next
// therefore grows at most linearly with the depth: the total is quadratic.
const SCROLL_PASSES: usize = 2;

fn subgrid_work_of_chain<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
) -> crate::accounting::SubgridWork {
    let host = crate::accounting::TestHost::new();
    crate::engine::compute_layout_invalidated_accounted(
        tree,
        0,
        standalone(),
        &[],
        Some(&host.scope()),
    )
    .expect("chain layout");
    host.subgrid_work()
}

/// The work of the chain at each of `depths`, on a stack that holds the native
/// recursion of the deepest chain.
fn subgrid_work_at_depths<S: LayoutScalar>(
    chain: impl Fn(u32) -> PublicLayoutTreeOf<S> + Send + 'static,
    depths: &'static [u32],
) -> Vec<crate::accounting::SubgridWork> {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            depths
                .iter()
                .map(|&depth| subgrid_work_of_chain(&chain(depth)))
                .collect()
        })
        .unwrap()
        .join()
        .unwrap()
}

/// Asserts that the contextual Grid computations grow from each depth to the
/// next by at most `allowed(depth)`.
fn assert_contextual_grids_bounded(
    name: &str,
    depths: &[u32],
    work: &[crate::accounting::SubgridWork],
    allowed: impl Fn(u32) -> usize,
) {
    for (&depth, pair) in depths.iter().zip(work.windows(2)) {
        let (lower, upper) = (pair[0].contextual_grids, pair[1].contextual_grids);
        assert!(
            upper <= lower + allowed(depth),
            "{name}: contextual Grid computations grow from {lower} at depth {depth} to {upper} \
             at depth {}, more than the {} one level may add (work by depth: {work:?})",
            depth + 1,
            allowed(depth),
        );
    }
}

const AUTO_COLUMN_WORK_DEPTHS: &[u32] = &[2, 3, 4, 5, 6, 7, 8];

#[test]
fn auto_column_chain_contextual_grids_grow_by_one_levels_requests_per_depth() {
    for work in [
        subgrid_work_at_depths::<f32>(
            |depth| auto_column_chain(depth, 40.0, false),
            AUTO_COLUMN_WORK_DEPTHS,
        ),
        subgrid_work_at_depths::<f64>(
            |depth| auto_column_chain(depth, 40.0, false),
            AUTO_COLUMN_WORK_DEPTHS,
        ),
    ] {
        assert_contextual_grids_bounded(
            "auto-column chain",
            AUTO_COLUMN_WORK_DEPTHS,
            &work,
            |_| OWN_ROW_REQUESTS_PER_LEVEL,
        );
    }
}

const AUTO_LANE_WORK_DEPTHS: &[u32] = &[2, 3, 4, 5, 6, 7, 8];

#[test]
fn auto_lane_chain_contextual_grids_grow_by_one_levels_requests_per_depth() {
    for work in [
        subgrid_work_at_depths::<f32>(auto_lane_chain, AUTO_LANE_WORK_DEPTHS),
        subgrid_work_at_depths::<f64>(auto_lane_chain, AUTO_LANE_WORK_DEPTHS),
    ] {
        assert_contextual_grids_bounded("auto-lane chain", AUTO_LANE_WORK_DEPTHS, &work, |_| {
            LANE_REQUESTS_PER_LEVEL
        });
    }
}

const SCROLLED_WORK_DEPTHS: &[u32] = &[2, 3, 4, 5, 6];

#[test]
fn scrolled_auto_column_chain_contextual_grids_grow_by_one_levels_requests_per_depth_below() {
    for work in [
        subgrid_work_at_depths::<f32>(
            |depth| auto_column_chain(depth, 60.0, true),
            SCROLLED_WORK_DEPTHS,
        ),
        subgrid_work_at_depths::<f64>(
            |depth| auto_column_chain(depth, 60.0, true),
            SCROLLED_WORK_DEPTHS,
        ),
    ] {
        assert_contextual_grids_bounded(
            "scrolled auto-column chain",
            SCROLLED_WORK_DEPTHS,
            &work,
            |depth| SCROLL_PASSES * OWN_ROW_REQUESTS_PER_LEVEL * (depth as usize + 1),
        );
    }
}

const ALTERNATING_WORK_DEPTHS: &[u32] = &[2, 3, 4, 5, 6, 7, 8, 9];

#[test]
fn alternating_chain_contextual_grids_grow_by_one_levels_requests_per_depth() {
    for work in [
        subgrid_work_at_depths::<f32>(alternating_chain, ALTERNATING_WORK_DEPTHS),
        subgrid_work_at_depths::<f64>(alternating_chain, ALTERNATING_WORK_DEPTHS),
    ] {
        assert_contextual_grids_bounded(
            "alternating chain",
            ALTERNATING_WORK_DEPTHS,
            &work,
            |_| OWN_ROW_REQUESTS_PER_LEVEL,
        );
    }
}

/// The debug rendering of the completed layout of `tree` and the work it
/// reported.
fn rendered_layout_and_work<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
) -> (String, crate::accounting::SubgridWork) {
    let host = crate::accounting::TestHost::new();
    let batch = crate::engine::compute_layout_invalidated_accounted(
        tree,
        0,
        standalone(),
        &[],
        Some(&host.scope()),
    )
    .expect("chain layout");
    (format!("{batch:?}"), host.subgrid_work())
}

/// Lays out `tree` with the reuse of subgrid computations and with every
/// computation repeated, and asserts that the published batch is the same and
/// that the reuse computed fewer contextual Grids.
fn assert_reuse_publishes_the_same_batch<S: LayoutScalar>(
    name: &str,
    tree: &PublicLayoutTreeOf<S>,
) {
    let (reused, reused_work) = rendered_layout_and_work(tree);
    let (repeated, repeated_work) =
        crate::grid::subgrid_work::with_subgrid_reuse_disabled(|| rendered_layout_and_work(tree));
    if reused != repeated {
        let at = reused
            .chars()
            .zip(repeated.chars())
            .position(|(left, right)| left != right)
            .unwrap_or_else(|| reused.len().min(repeated.len()));
        let window = |text: &str| {
            text.chars()
                .skip(at.saturating_sub(150))
                .take(400)
                .collect::<String>()
        };
        panic!(
            "{name}: reuse changed the published batch, first at {at}: reused {} repeated {}",
            window(&reused),
            window(&repeated)
        );
    }
    assert!(
        reused_work.contextual_grids < repeated_work.contextual_grids,
        "{name}: the reuse engages: {reused_work:?} against {repeated_work:?}"
    );
}

/// Runs `check` on a stack that holds the native recursion of the repeated
/// layout, which computes every path through the chain.
fn on_large_stack(check: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(check)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn auto_column_chain_publishes_the_same_batch_with_and_without_reuse() {
    on_large_stack(|| {
        assert_reuse_publishes_the_same_batch(
            "auto-column chain f32",
            &auto_column_chain::<f32>(3, 40.0, false),
        );
        assert_reuse_publishes_the_same_batch(
            "auto-column chain f64",
            &auto_column_chain::<f64>(3, 40.0, false),
        );
    });
}

#[test]
fn auto_lane_chain_publishes_the_same_batch_with_and_without_reuse() {
    on_large_stack(|| {
        assert_reuse_publishes_the_same_batch("auto-lane chain f32", &auto_lane_chain::<f32>(3));
        assert_reuse_publishes_the_same_batch("auto-lane chain f64", &auto_lane_chain::<f64>(3));
    });
}

#[test]
fn scrolled_auto_column_chain_publishes_the_same_batch_with_and_without_reuse() {
    on_large_stack(|| {
        assert_reuse_publishes_the_same_batch(
            "scrolled auto-column chain f32",
            &auto_column_chain::<f32>(2, 60.0, true),
        );
        assert_reuse_publishes_the_same_batch(
            "scrolled auto-column chain f64",
            &auto_column_chain::<f64>(2, 60.0, true),
        );
    });
}

#[test]
fn alternating_chain_publishes_the_same_batch_with_and_without_reuse() {
    on_large_stack(|| {
        assert_reuse_publishes_the_same_batch(
            "alternating chain f32",
            &alternating_chain::<f32>(4),
        );
        assert_reuse_publishes_the_same_batch(
            "alternating chain f64",
            &alternating_chain::<f64>(4),
        );
    });
}

// Chains that differ from the auto-column chain in one fact the sizes of the
// innermost level depend on. A request a level makes of the level below is
// answered for the tracks, gap and percentage basis it was made with, so a
// layout that reuses one answer for another request gives the wrong geometry.
// Node 0 is a Grid with two columns and one automatic row; nodes 1..=depth
// are Grid items that take both columns as subgrid columns and have two
// automatic rows of their own, as in `auto_column_chain`. The last level
// holds `depth + 1`, 20px high in its first column and row with width
// `leaf_width`, and `depth + 2`, 40x30 in the second of each.
fn own_row_chain<S: LayoutScalar>(
    depth: u32,
    columns: [TrackComponentOf<S>; 2],
    root_width: f64,
    column_gap: f64,
    leaf_width: crate::sizing::PreferredSizeOf<S>,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: px_size(root_width, 40.0),
            grid_template_columns: columns.into(),
            grid_template_rows: vec![TrackComponentOf::AUTO],
            gap: Size::new(
                crate::length::LengthOf::px(S::from_f64(column_gap)),
                crate::length::LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: lines(1, 3),
                    grid_row: lines(1, 2),
                    align_self: Some(AlignItems::Start),
                    grid_template_columns: subgrid(),
                    grid_template_rows: auto_tracks(),
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    tree = tree
        .style(
            depth + 1,
            NodeInputOf {
                grid_column: line(1),
                grid_row: line(1),
                size: Size::new(
                    leaf_width,
                    crate::sizing::PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(depth + 2, leaf(2, 2, 40.0, 30.0));
    tree.children(depth, [depth + 1, depth + 2])
}

fn assert_own_row_chain<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    depth: u32,
    root_width: f64,
    first_leaf: (f64, f64),
    second_leaf_x: f64,
) {
    let batch = layout_of(tree, depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (root_width, 40.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (root_width, 50.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), first_leaf);
    assert_box(&batch, depth + 2, (second_leaf_x, 20.0), (40.0, 30.0));
}

// CSS Grid Level 1, section 11.8: free space is distributed equally to the
// automatic tracks. The leaves, 30px and 40px wide, and the 10px column gap,
// leave 90 - 30 - 40 - 10 = 10px, so the columns are 35px and 45px and the
// second one starts at 35 + 10 = 45px. Every level spans both columns, and so
// is 35 + 10 + 45 = 90px wide. A subgrid's gap is that of its parent (Grid
// Level 2, section 9), and the levels' own rows size to 20 + 30 = 50px.
#[test]
fn own_row_chain_with_a_column_gap_places_the_second_column_after_the_gap() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::own_row_chain_with_a_column_gap_places_the_second_column_after_the_gap",
        || {
            let chain = |depth| {
                own_row_chain::<f32>(
                    depth,
                    [TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                    90.0,
                    10.0,
                    crate::sizing::PreferredSizeOf::px(30.0),
                )
            };
            assert_own_row_chain(&chain(3), 3, 90.0, (30.0, 20.0), 45.0);
            let chain = |depth| {
                own_row_chain::<f64>(
                    depth,
                    [TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                    90.0,
                    10.0,
                    crate::sizing::PreferredSizeOf::px(30.0),
                )
            };
            assert_own_row_chain(&chain(3), 3, 90.0, (30.0, 20.0), 45.0);
        },
    );
}

// CSS Grid Level 1, section 6.2 and 11.8: the automatic column takes the
// 100 - 40 = 60px the fixed column leaves, and a percentage width resolves
// against the grid area, so the first leaf is 50% of 60 = 30px wide while the
// automatic column is sized as if the percentage were auto. The second column
// starts at 60px.
#[test]
fn own_row_chain_with_a_percentage_leaf_resolves_it_against_the_final_column() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::own_row_chain_with_a_percentage_leaf_resolves_it_against_the_final_column",
        || {
            let tree = own_row_chain::<f32>(
                3,
                [TrackComponentOf::AUTO, TrackComponentOf::px(40.0)],
                100.0,
                0.0,
                crate::sizing::PreferredSizeOf::percent(0.5),
            );
            assert_own_row_chain(&tree, 3, 100.0, (30.0, 20.0), 60.0);
            let tree = own_row_chain::<f64>(
                3,
                [TrackComponentOf::AUTO, TrackComponentOf::px(40.0)],
                100.0,
                0.0,
                crate::sizing::PreferredSizeOf::percent(0.5),
            );
            assert_own_row_chain(&tree, 3, 100.0, (30.0, 20.0), 60.0);
        },
    );
}

// The columns are fixed at 20px and 50px, narrower than the first leaf, which
// is 30px wide and overflows its track. The second column starts at 20px, and
// every level is 20 + 50 = 70px wide.
#[test]
fn own_row_chain_with_fixed_columns_places_the_second_column_at_the_first_tracks_end() {
    on_bounded_stack(
        "engine::depth_tests::contextual_intrinsic::own_row_chain_with_fixed_columns_places_the_second_column_at_the_first_tracks_end",
        || {
            let tree = own_row_chain::<f32>(
                3,
                [TrackComponentOf::px(20.0), TrackComponentOf::px(50.0)],
                70.0,
                0.0,
                crate::sizing::PreferredSizeOf::px(30.0),
            );
            assert_own_row_chain(&tree, 3, 70.0, (30.0, 20.0), 20.0);
            let tree = own_row_chain::<f64>(
                3,
                [TrackComponentOf::px(20.0), TrackComponentOf::px(50.0)],
                70.0,
                0.0,
                crate::sizing::PreferredSizeOf::px(30.0),
            );
            assert_own_row_chain(&tree, 3, 70.0, (30.0, 20.0), 20.0);
        },
    );
}
