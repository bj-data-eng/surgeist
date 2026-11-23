use super::contextual_grid_children::{assert_box, px_size, subgrid};
use super::*;
use crate::{
    grid::GridAxisKind,
    node_input::{
        AlignContent, ComputedOverflow, GridLanesFlow, GridPlacement, Overflow, ScrollbarWidthOf,
    },
    value::TrackComponentOf,
};

// Chains whose items size intrinsic tracks of their parent. Every level is an
// ordinary item of the level above, so its intrinsic contribution is a Grid
// computed with its parent's tracks as context. Each level costs several
// hundred KiB of native stack in a debug build, f64 more than f32, and
// computes its subtree again for each intrinsic query, so the work grows
// geometrically with the depth. The deep depths below overflow the 2 MiB stack
// with the margin the comment on each constant gives, and the whole layout
// still finishes in seconds once the recursion is off the stack. A scalar
// whose frames overflow only at a depth the geometric work makes impractical
// has no deep test for that chain.

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

// f64 overflows from 4 levels and f32 from 7.
const AUTO_COLUMN_DEPTH_F32: u32 = 8;
const AUTO_COLUMN_DEPTH_F64: u32 = 6;

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
// f64 overflows from 3 levels. The f32 chain does not overflow within the 5
// levels that finish quickly.
const SCROLLED_COLUMN_DEPTH: u32 = 4;

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

// f64 overflows from 8 levels. The f32 chain does not overflow within the 12
// levels that finish quickly.
const ALTERNATING_DEPTH: u32 = 10;

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

// f64 overflows from 4 levels. f32 overflows from 7, where the work is too
// large for a quick test.
const AUTO_LANE_DEPTH: u32 = 6;

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

// f64 overflows from 5 levels. f32 overflows from 8, where the work is too
// large for a quick test.
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
