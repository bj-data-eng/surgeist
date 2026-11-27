use super::scoped_flex::{boxes, scoped_layout};
use super::*;
use crate::{
    geometry::Point,
    node_input::{Float, SizeContainment},
    sizing::PreferredSizeOf,
};

/// A size-contained 60x40px Block (node 1, an atomic box for fragmentation)
/// holding a chain of same-axis Block containers. Node 0 is the 60px-wide root
/// and node `depth` the leaf, a 3px-tall empty box. Every container above the
/// leaf has a preceding left float sibling of 60x1px, so each in-flow Block
/// child shares the float context of its parent. The float of spine node `k`
/// is node `depth + k`.
pub(super) fn assert_float_sibling_spine<S: LayoutScalar>(depth: u32) {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let mut tree = PublicLayoutTreeOf::<S>::new();
    for node in 0..=depth {
        let (size, size_containment) = match node {
            0 => (
                Size::new(px(60.0), PreferredSizeOf::AUTO),
                SizeContainment::None,
            ),
            1 => (Size::new(px(60.0), px(40.0)), SizeContainment::BothAxes),
            _ if node == depth => (
                Size::new(PreferredSizeOf::AUTO, px(3.0)),
                SizeContainment::None,
            ),
            _ => (
                Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                SizeContainment::None,
            ),
        };
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Block,
                size,
                size_containment,
                ..NodeInputOf::default()
            },
        );
        if node < depth {
            if node == 0 {
                tree = tree.children(0, [1]);
                continue;
            }
            let float = depth + node;
            tree = tree
                .style(
                    float,
                    NodeInputOf {
                        display: Display::Block,
                        float: Float::Left,
                        size: Size::new(px(60.0), px(1.0)),
                        ..NodeInputOf::default()
                    },
                )
                .children(node, [float, node + 1]);
        }
    }
    let nodes = 2 * depth;
    let batch = scoped_layout(&tree, nodes);
    let geometry = boxes(&batch, nodes);
    // The size-contained box keeps its specified 60x40px size and the root
    // takes its height (CSS Box Sizing 4 size containment; CSS 2.1 10.6.3).
    // Descendant automatic-width blocks fill the 60px content box (CSS 2.1
    // 10.3.3). A block that does not establish a formatting context has an
    // automatic height equal to its single in-flow 3px leaf and ignores floats
    // (CSS 2.1 10.6.3), and each in-flow block box starts at its parent's
    // origin because a float leaves normal flow (CSS 2.1 9.5, 9.5.1). Each
    // 60px float fills the containing block width, so rule 2 of 9.5.1 puts
    // the float of spine node `k` below the float of node `k - 1`, at y = k - 1.
    for node in 0..=depth {
        let (width, height) = match node {
            0 | 1 => (60.0, 40.0),
            _ => (60.0, 3.0),
        };
        assert_eq!(
            geometry[node as usize].size,
            Size::new(S::from_f64(width), S::from_f64(height)),
            "node {node}"
        );
        assert_eq!(geometry[node as usize].location, Point::ZERO, "node {node}");
    }
    for node in 1..depth {
        let float = (depth + node) as usize;
        assert_eq!(
            geometry[float].size,
            Size::new(S::from_f64(60.0), S::from_f64(1.0)),
            "float of node {node}"
        );
        assert_eq!(
            geometry[float].location,
            Point::new(S::ZERO, S::from_f64(f64::from(node - 1))),
            "float of node {node}"
        );
    }
}

#[test]
fn shallow_float_sibling_block_spine_lays_out_under_block_fragmentation() {
    on_bounded_stack(
        "engine::depth_tests::scoped_float_spine::shallow_float_sibling_block_spine_lays_out_under_block_fragmentation",
        || {
            assert_float_sibling_spine::<f32>(3);
            assert_float_sibling_spine::<f64>(3);
        },
    );
}

#[test]
fn deep_float_sibling_block_spine_lays_out_under_block_fragmentation() {
    on_bounded_stack(
        "engine::depth_tests::scoped_float_spine::deep_float_sibling_block_spine_lays_out_under_block_fragmentation",
        || {
            assert_float_sibling_spine::<f32>(24);
            assert_float_sibling_spine::<f64>(24);
        },
    );
}
