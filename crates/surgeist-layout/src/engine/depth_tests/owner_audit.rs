use super::contextual_grid_children::{assert_box, px_size, subgrid};
use super::*;
use crate::{
    node_input::{AlignContent, AlignItems, FlexDirection, GridPlacement},
    value::TrackComponentOf,
};

// The live browser-parity corpus holds supported inputs of seven source box
// levels (the deepest, `android_news_feed`, nests seven), so a tree of that
// depth lays out with no depth refusal and with the geometry its layout
// algorithms define.
//
// Node 0 is a Block, node 1 a row Flex container, node 2 a Grid with two fixed
// 30px columns, node 3 a Block in the second column, node 4 a column Flex
// container, node 5 a 20x10 Block and node 6 a 5x4 Block.
fn seven_level_tree<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    PublicLayoutTreeOf::<S>::new()
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
                display: Display::Flex,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(px(60.0), PreferredSizeOf::AUTO),
                grid_template_columns: vec![
                    TrackComponentOf::px(S::from_f64(30.0)),
                    TrackComponentOf::px(S::from_f64(30.0)),
                ],
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                grid_column: GridPlacement::try_line(2).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: Some(AlignItems::Start),
                justify_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(px(20.0), px(10.0)),
                ..NodeInputOf::default()
            },
        )
        .style(
            6,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(px(5.0), px(4.0)),
                ..NodeInputOf::default()
            },
        )
        .children(0, [1])
        .children(1, [2])
        .children(2, [3])
        .children(3, [4])
        .children(4, [5])
        .children(5, [6])
}

// CSS Flexbox 9.4: the Flex container's cross size is its tallest item, and
// its stretched item fills it. CSS Grid 6.1 and 11: the automatic row takes the
// height of the item in it, which a Block sizes to its in-flow content (CSS 2
// 10.6.3). A Block with an automatic width fills its containing block (CSS 2
// 10.3.3), a Flex or Grid item with `align-self: start` shrinks to its width.
// The root is 100px wide, so every level of height 10 is the 10px Block in the
// innermost flex.
fn assert_seven_level_ordinary_layout<S: LayoutScalar>() {
    let tree = seven_level_tree::<S>();
    let batch = compute_layout(&tree, 0, standalone()).expect("seven-level tree lays out");
    assert_eq!(batch.node_layouts().len(), 7);
    assert_box(&batch, 0, (0.0, 0.0), (100.0, 10.0));
    assert_box(&batch, 1, (0.0, 0.0), (100.0, 10.0));
    assert_box(&batch, 2, (0.0, 0.0), (60.0, 10.0));
    assert_box(&batch, 3, (30.0, 0.0), (30.0, 10.0));
    assert_box(&batch, 4, (0.0, 0.0), (30.0, 10.0));
    assert_box(&batch, 5, (0.0, 0.0), (20.0, 10.0));
    assert_box(&batch, 6, (0.0, 0.0), (5.0, 4.0));
}

#[test]
fn seven_level_mixed_source_tree_lays_out_unchanged() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::seven_level_mixed_source_tree_lays_out_unchanged",
        || {
            assert_seven_level_ordinary_layout::<f32>();
            assert_seven_level_ordinary_layout::<f64>();
        },
    );
}

// The same depth under Block fragmentation: a spine of seven source boxes
// below the root, with a float sibling at each level, lays out through the
// scoped Block descent.
#[test]
fn seven_level_block_spine_lays_out_unchanged_under_block_fragmentation() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::seven_level_block_spine_lays_out_unchanged_under_block_fragmentation",
        || {
            super::scoped_float_spine::assert_float_sibling_spine::<f32>(6);
            super::scoped_float_spine::assert_float_sibling_spine::<f64>(6);
        },
    );
}

fn line(number: isize) -> GridPlacement {
    GridPlacement::try_line(number).unwrap()
}

// A Grid chain whose levels each size their automatic columns from a subgrid
// wrapper and a baseline-aligned item. Level `k` is the Grid `3k` with two
// automatic columns and one automatic row. Node `3k + 1` is its subgrid
// wrapper, a subgrid in the columns in the first column, holding the 10x5 leaf
// `3k + 2`. Node `3k + 3` is the next level, the Grid in the second column with
// `justify-self: baseline`. The innermost level, `3 * depth`, holds only its
// wrapper. With `rows_subgrid`, every level below the root is also a subgrid
// in the rows.
fn baseline_chain<S: LayoutScalar>(depth: u32, rows_subgrid: bool) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::<S>::new();
    for level in 0..=depth {
        let grid = 3 * level;
        let mut style = NodeInputOf {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
            grid_template_rows: vec![TrackComponentOf::AUTO],
            ..NodeInputOf::default()
        };
        if level == 0 {
            style.size = Size::new(
                PreferredSizeOf::px(S::from_f64(10.0 * f64::from(depth + 1))),
                PreferredSizeOf::AUTO,
            );
        } else {
            style.grid_column = line(2);
            style.grid_row = line(1);
            style.justify_self = Some(AlignItems::Baseline);
            if rows_subgrid {
                style.grid_template_rows = subgrid();
            }
        }
        tree = tree
            .style(grid, style)
            .style(
                grid + 1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: line(1),
                    grid_row: line(1),
                    grid_template_columns: subgrid(),
                    ..NodeInputOf::default()
                },
            )
            .style(
                grid + 2,
                NodeInputOf {
                    grid_column: line(1),
                    grid_row: line(1),
                    size: px_size(10.0, 5.0),
                    ..NodeInputOf::default()
                },
            )
            .children(grid + 1, [grid + 2]);
        tree = if level < depth {
            tree.children(grid, [grid + 1, grid + 3])
        } else {
            tree.children(grid, [grid + 1])
        };
    }
    tree
}

// CSS Grid Level 2, section 9: a subgrid contributes its items to the tracks it
// spans, so the first column of every level is the 10px of its leaf. The second
// column is the max-content width of the next level, which is its own two
// columns, so a level holds `10 * (levels at and below it)` px. The root is
// given that width. A baseline-aligned item that shares no column with another
// takes its start position, and the automatic row is the 5px of the leaves.
fn assert_baseline_chain<S: LayoutScalar>(depth: u32, rows_subgrid: bool) {
    let tree = baseline_chain::<S>(depth, rows_subgrid);
    let batch = compute_layout(&tree, 0, standalone()).expect("baseline chain lays out");
    assert_eq!(batch.node_layouts().len(), 3 * depth as usize + 3);
    for level in 0..=depth {
        let grid = 3 * level;
        let width = 10.0 * f64::from(depth + 1 - level);
        let at = if level == 0 { (0.0, 0.0) } else { (10.0, 0.0) };
        assert_box(&batch, grid, at, (width, 5.0));
        assert_box(&batch, grid + 1, (0.0, 0.0), (10.0, 5.0));
        assert_box(&batch, grid + 2, (0.0, 0.0), (10.0, 5.0));
    }
}

// Each level nests a driver on the native stack, and the work of the chain
// grows geometrically with the depth (#2883), so the depths are the smallest
// past the first stack overflow of each scalar type on a 2 MiB stack.
const BASELINE_COLUMN_DEPTH_F32: u32 = 6;
const BASELINE_COLUMN_DEPTH_F64: u32 = 3;
// With subgrid rows the work grows geometrically and steeply: the f32 chain
// takes 20 s at three levels and 316 s at four in a debug build without
// overflowing the stack, so only the f64 chain, which overflows at three levels,
// is checked deep.
const BASELINE_ROW_DEPTH_F64: u32 = 3;

#[test]
fn shallow_baseline_aligned_item_chain_sizes_columns_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::shallow_baseline_aligned_item_chain_sizes_columns_from_leaves",
        || {
            assert_baseline_chain::<f32>(2, false);
            assert_baseline_chain::<f64>(2, false);
        },
    );
}

#[test]
fn deep_baseline_aligned_item_chain_sizes_columns_from_leaves_f32() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::deep_baseline_aligned_item_chain_sizes_columns_from_leaves_f32",
        || assert_baseline_chain::<f32>(BASELINE_COLUMN_DEPTH_F32, false),
    );
}

#[test]
fn deep_baseline_aligned_item_chain_sizes_columns_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::deep_baseline_aligned_item_chain_sizes_columns_from_leaves_f64",
        || assert_baseline_chain::<f64>(BASELINE_COLUMN_DEPTH_F64, false),
    );
}

#[test]
fn shallow_baseline_aligned_row_subgrid_chain_sizes_columns_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::shallow_baseline_aligned_row_subgrid_chain_sizes_columns_from_leaves",
        || {
            assert_baseline_chain::<f32>(2, true);
            assert_baseline_chain::<f64>(2, true);
        },
    );
}

#[test]
fn deep_baseline_aligned_row_subgrid_chain_sizes_columns_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::deep_baseline_aligned_row_subgrid_chain_sizes_columns_from_leaves_f64",
        || assert_baseline_chain::<f64>(BASELINE_ROW_DEPTH_F64, true),
    );
}

// The collector recurses natively and its work is linear in the depth. The
// first overflow of a 2 MiB stack is at 30 levels (f64) and 50 levels (f32).
const NESTED_LANE_DEPTH: u32 = 100;

#[test]
fn deep_nested_lane_wrapper_chain_sizes_automatic_lanes_from_leaves_f32() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::deep_nested_lane_wrapper_chain_sizes_automatic_lanes_from_leaves_f32",
        || super::contextual_intrinsic::assert_auto_lane_chain::<f32>(NESTED_LANE_DEPTH),
    );
}

#[test]
fn deep_nested_lane_wrapper_chain_sizes_automatic_lanes_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::deep_nested_lane_wrapper_chain_sizes_automatic_lanes_from_leaves_f64",
        || super::contextual_intrinsic::assert_auto_lane_chain::<f64>(NESTED_LANE_DEPTH),
    );
}

// Grid Lanes chain of scroll containers with automatic lanes. Node 0 is a 40x14
// Grid Lanes container with two automatic lanes in the column axis. Nodes
// 1..=depth are Grid Lanes subgrids in the lane axis, each the only item of its
// parent and spanning both lanes. The last holds three leaves, 20px wide and
// 5px, 7px and 9px high. Every container has `overflow: auto`, so each one
// observes its children's overflow for its scrollbars.
fn scrolled_lane_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let flow =
        crate::node_input::GridLanesFlow::new(crate::grid::GridAxisKind::Column, false, false);
    let scroll = |mut style: NodeInputOf<S>| {
        style.overflow = crate::node_input::ComputedOverflow::try_new(
            crate::node_input::Overflow::Auto,
            crate::node_input::Overflow::Auto,
        )
        .unwrap();
        style.scrollbar_width =
            crate::node_input::ScrollbarWidthOf::try_new(S::from_f64(15.0)).unwrap();
        style
    };
    let mut tree = PublicLayoutTreeOf::<S>::new().style(
        0,
        scroll(NodeInputOf {
            display: Display::GridLanes,
            grid_lanes_flow: flow,
            size: px_size(40.0, 14.0),
            grid_template_columns: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        }),
    );
    for node in 1..=depth {
        tree = tree
            .style(
                node,
                scroll(NodeInputOf {
                    display: Display::GridLanes,
                    grid_lanes_flow: flow,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: subgrid(),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                }),
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

// CSS Grid Level 3, Grid Lanes: each item goes to the lane with the least used
// extent, the first on ties, so the 5px and 7px items take the two lanes and
// the 9px item goes below the 5px item; the items of a subgrid size the
// parent's automatic lanes, 20px each. The content of every level is 40x14, the
// size of its box, so no scrollbar is needed (CSS Overflow 3, `auto`).
fn assert_scrolled_lane_chain<S: LayoutScalar>(depth: u32) {
    let tree = scrolled_lane_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("scrolled lane chain lays out");
    assert_eq!(batch.node_layouts().len(), depth as usize + 4);
    for node in 0..=depth {
        assert_box(&batch, node, (0.0, 0.0), (40.0, 14.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (20.0, 5.0));
    assert_box(&batch, depth + 2, (20.0, 0.0), (20.0, 7.0));
    assert_box(&batch, depth + 3, (0.0, 5.0), (20.0, 9.0));
}

// The work is linear in the depth. The first overflow of a 2 MiB stack is at 4
// levels (f64) and 6 levels (f32).
const SCROLLED_LANE_DEPTH: u32 = 24;

#[test]
fn shallow_scrolled_lane_chain_observes_overflow_and_sizes_lanes_from_leaves() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::shallow_scrolled_lane_chain_observes_overflow_and_sizes_lanes_from_leaves",
        || {
            assert_scrolled_lane_chain::<f32>(3);
            assert_scrolled_lane_chain::<f64>(3);
        },
    );
}

#[test]
fn deep_scrolled_lane_chain_observes_overflow_and_sizes_lanes_from_leaves_f32() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::deep_scrolled_lane_chain_observes_overflow_and_sizes_lanes_from_leaves_f32",
        || assert_scrolled_lane_chain::<f32>(SCROLLED_LANE_DEPTH),
    );
}

#[test]
fn deep_scrolled_lane_chain_observes_overflow_and_sizes_lanes_from_leaves_f64() {
    on_bounded_stack(
        "engine::depth_tests::owner_audit::deep_scrolled_lane_chain_observes_overflow_and_sizes_lanes_from_leaves_f64",
        || assert_scrolled_lane_chain::<f64>(SCROLLED_LANE_DEPTH),
    );
}
