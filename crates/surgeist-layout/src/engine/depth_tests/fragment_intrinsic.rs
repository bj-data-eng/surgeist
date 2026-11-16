use super::scoped_flex::{boxes, scoped_layout};
use super::*;
use crate::{Edges, LengthOf, Point, PreferredSizeOf};

/// A chain of same-axis Block containers under one block-fragmentation
/// request. Node 0 is the 60px-wide root and node `depth` is the leaf, a
/// 20px-tall empty box. Every container between them carries 2px of left
/// padding and automatic size.
fn assert_same_axis_block_spine<S: LayoutScalar>(depth: u32) {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let mut tree = PublicLayoutTreeOf::<S>::new();
    for node in 0..=depth {
        let style = if node == 0 {
            NodeInputOf {
                display: Display::Block,
                size: Size::new(px(60.0), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            }
        } else if node == depth {
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::AUTO, px(20.0)),
                ..NodeInputOf::default()
            }
        } else {
            NodeInputOf {
                display: Display::Block,
                padding: Edges::new(
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    LengthOf::px(S::from_f64(2.0)),
                ),
                ..NodeInputOf::default()
            }
        };
        tree = tree.style(node, style);
        if node < depth {
            tree = tree.children(node, [node + 1]);
        }
    }
    let nodes = depth + 1;
    let batch = scoped_layout(&tree, nodes);
    let geometry = boxes(&batch, nodes);
    for node in 0..nodes {
        // An automatic-width block fills its containing block's content box
        // (CSS 2.1 10.3.3). Each padded ancestor below the root narrows its
        // content box by 2px, and each child starts at its parent's 2px left
        // padding edge (CSS 2.1 8.4). Automatic heights follow the single
        // 20px in-flow leaf (CSS 2.1 10.6.3). Nothing breaks inside the one
        // 100px page frame (CSS Fragmentation 3, section 4).
        let width = 60.0 - 2.0 * f64::from(node.saturating_sub(1));
        let x = if node >= 2 { 2.0 } else { 0.0 };
        assert_eq!(
            geometry[node as usize].size,
            Size::new(S::from_f64(width), S::from_f64(20.0)),
            "node {node}"
        );
        assert_eq!(
            geometry[node as usize].location,
            Point::new(S::from_f64(x), S::ZERO),
            "node {node}"
        );
    }
}

#[test]
fn shallow_same_axis_block_spine_lays_out_under_block_fragmentation() {
    on_bounded_stack(
        "engine::depth_tests::fragment_intrinsic::shallow_same_axis_block_spine_lays_out_under_block_fragmentation",
        || {
            assert_same_axis_block_spine::<f32>(3);
            assert_same_axis_block_spine::<f64>(3);
        },
    );
}

#[test]
fn deep_same_axis_block_spine_lays_out_under_block_fragmentation() {
    on_bounded_stack(
        "engine::depth_tests::fragment_intrinsic::deep_same_axis_block_spine_lays_out_under_block_fragmentation",
        || {
            assert_same_axis_block_spine::<f32>(24);
            assert_same_axis_block_spine::<f64>(24);
        },
    );
}

/// An unpadded chain of same-axis Block containers: node 0 is the 60px-wide
/// root, node `depth` the 20px-tall empty leaf, and every box between has
/// automatic size.
fn assert_unpadded_block_spine<S: LayoutScalar>(depth: u32) {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let mut tree = PublicLayoutTreeOf::<S>::new();
    for node in 0..=depth {
        let size = match node {
            0 => Size::new(px(60.0), PreferredSizeOf::AUTO),
            _ if node == depth => Size::new(PreferredSizeOf::AUTO, px(20.0)),
            _ => Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
        };
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Block,
                size,
                ..NodeInputOf::default()
            },
        );
        if node < depth {
            tree = tree.children(node, [node + 1]);
        }
    }
    let nodes = depth + 1;
    let batch = scoped_layout(&tree, nodes);
    let geometry = boxes(&batch, nodes);
    // Every automatic-width block fills the root's 60px content box (CSS 2.1
    // 10.3.3), every automatic height is the single 20px leaf (CSS 2.1
    // 10.6.3), and with no padding, border or margin each box starts at its
    // parent's origin (CSS 2.1 8.1). Nothing breaks inside the one 100px page
    // frame (CSS Fragmentation 3, section 4).
    for node in 0..nodes {
        assert_eq!(
            geometry[node as usize].size,
            Size::new(S::from_f64(60.0), S::from_f64(20.0)),
            "node {node}"
        );
        assert_eq!(geometry[node as usize].location, Point::ZERO, "node {node}");
    }
}

#[test]
fn very_deep_same_axis_block_spine_lays_out_under_block_fragmentation() {
    on_bounded_stack(
        "engine::depth_tests::fragment_intrinsic::very_deep_same_axis_block_spine_lays_out_under_block_fragmentation",
        || {
            assert_unpadded_block_spine::<f32>(1000);
            assert_unpadded_block_spine::<f64>(1000);
        },
    );
}
