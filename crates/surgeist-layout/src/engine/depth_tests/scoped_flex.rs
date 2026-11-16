use super::*;
use crate::{
    AlignItems, AspectRatioOf, BlockFragmentainerOf, FlexDirection, FragmentationBreakKinds,
    FragmentationKind, LayoutBoundRequestOf, LayoutFlowInput, LayoutSourceBasis, MinSizeOf,
    PageProgression, Point, PreferredSizeOf, WrappingInput, compute_layout_with_flow,
};

/// Lays out `tree` under a one-page block fragmentation request, with a
/// 60px-wide root and a page frame tall enough to hold every node.
pub(super) fn scoped_layout<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    nodes: u32,
) -> crate::CompletedLayoutBatchOf<u32, S> {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let sizing = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(60.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let request = LayoutRootRequestOf::standalone(sizing, axes, Size::NONE);
    let bound = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        request,
        FragmentationKind::Page {
            progression: PageProgression::LeftToRight,
        },
        axes,
        vec![BlockFragmentainerOf {
            origin: Point::ZERO,
            size: Size::new(S::from_f64(60.0), S::from_f64(100.0)),
            breaks: FragmentationBreakKinds::NONE,
            page_side: None,
        }],
    )
    .expect("one page frame");
    let target = bound.target_basis();
    let flow = LayoutFlowInput::try_new_bound(
        tree,
        0,
        bound,
        Vec::new(),
        WrappingInput::default(),
        Vec::new(),
    )
    .expect("bound numeric flow");
    let batch = compute_layout_with_flow(&flow, 0, request).expect("scoped flex chain returns");
    assert_eq!(batch.target_basis(), Some(&target));
    assert_eq!(batch.node_layouts().len(), nodes as usize);
    assert_eq!(batch.fragmentainers().len(), 1);
    batch
}

/// The geometry of the one actual box published for each node.
pub(super) fn boxes<S: LayoutScalar>(
    batch: &crate::CompletedLayoutBatchOf<u32, S>,
    nodes: u32,
) -> Vec<crate::BoxFragmentGeometryOf<S>> {
    let mut found = vec![None; nodes as usize];
    for fragment in batch.fragments() {
        let slot = &mut found[fragment.node() as usize];
        assert!(
            slot.is_none(),
            "one actual box for node {}",
            fragment.node()
        );
        *slot = Some(
            fragment
                .principal_box()
                .expect("actual box")
                .final_geometry(),
        );
    }
    found
        .into_iter()
        .enumerate()
        .map(|(node, geometry)| geometry.unwrap_or_else(|| panic!("box for node {node}")))
        .collect()
}

/// An orthogonal atomic box holding a chain of Flex containers, rooted in a
/// Block. Nodes 0 and 1 are the root and the atomic box; the chain starts at
/// node 2 and ends at the leaf, node `depth + 2`.
fn scoped_chain_tree<S: LayoutScalar>(
    atomic_size: Size<PreferredSizeOf<S>>,
    depth: u32,
    chain_style: impl Fn(u32, u32) -> NodeInputOf<S>,
) -> PublicLayoutTreeOf<S> {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(px(60.0), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: atomic_size,
                ..NodeInputOf::default()
            },
        );
    let leaf = depth + 2;
    for node in 2..=leaf {
        tree = tree.style(node, chain_style(node, leaf));
        if node < leaf {
            tree = tree.children(node, [node + 1]);
        }
    }
    tree
}

/// Each Flex container has a 120px item that shrinks to the 60px container.
/// The leaf's automatic height follows its 2:1 ratio at the shrunk width, and
/// every automatic container height takes the height of its sole item.
fn assert_scoped_shrinking_auto_cross_chain<S: LayoutScalar>(depth: u32) {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let tree = scoped_chain_tree::<S>(Size::new(px(60.0), px(40.0)), depth, |node, leaf| {
        let is_leaf = node == leaf;
        NodeInputOf {
            display: if is_leaf {
                Display::Block
            } else {
                Display::Flex
            },
            size: Size::new(
                px(if node == 2 { 60.0 } else { 120.0 }),
                PreferredSizeOf::AUTO,
            ),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
            aspect_ratio: if is_leaf {
                AspectRatioOf::new(S::from_f64(2.0))
            } else {
                None
            },
            align_items: Some(AlignItems::Start),
            ..NodeInputOf::default()
        }
    });
    let nodes = depth + 3;
    let batch = scoped_layout(&tree, nodes);
    let geometry = boxes(&batch, nodes);
    // The atomic box is a fixed 60x40 and the chain descends from its origin.
    assert_eq!(
        geometry[1].size,
        Size::new(S::from_f64(60.0), S::from_f64(40.0))
    );
    assert_eq!(geometry[1].location, Point::ZERO);
    for node in 2..nodes {
        // A 120px item shrinks to its 60px container (CSS Flexbox 9.7); the
        // 60px leaf width gives a 30px height through the 2:1 ratio, and each
        // container's used cross size is its sole line's 30px cross size (CSS
        // Flexbox 9.4, steps 7 and 8 and the container's used cross size).
        assert_eq!(
            geometry[node as usize].size,
            Size::new(S::from_f64(60.0), S::from_f64(30.0)),
            "node {node}"
        );
        assert_eq!(geometry[node as usize].location, Point::ZERO, "node {node}");
    }
}

#[test]
fn shallow_scoped_flex_chain_sizes_automatic_cross_axis_from_shrunk_items() {
    on_bounded_stack(
        "engine::depth_tests::scoped_flex::shallow_scoped_flex_chain_sizes_automatic_cross_axis_from_shrunk_items",
        || {
            assert_scoped_shrinking_auto_cross_chain::<f32>(2);
            assert_scoped_shrinking_auto_cross_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_scoped_flex_chain_sizes_automatic_cross_axis_from_shrunk_items() {
    on_bounded_stack(
        "engine::depth_tests::scoped_flex::deep_scoped_flex_chain_sizes_automatic_cross_axis_from_shrunk_items",
        || {
            assert_scoped_shrinking_auto_cross_chain::<f32>(24);
            assert_scoped_shrinking_auto_cross_chain::<f64>(24);
        },
    );
}

/// Node 2 aligns its two items by baseline. Its first item is a chain of
/// 30x40 Flex containers ending in an empty 30x20 Block leaf, so each
/// container's first baseline is the leaf's 20px bottom edge (CSS Flexbox
/// 8.5). The second item is an empty 30x30 Block with a 30px baseline. The
/// deeper baseline places the first item 10px below the line start (CSS
/// Flexbox 9.4 step 8).
fn assert_scoped_baseline_alignment_chain<S: LayoutScalar>(depth: u32) {
    let px = |value| PreferredSizeOf::px(S::from_f64(value));
    let leaf = depth + 2;
    let sibling = leaf + 1;
    let mut tree =
        scoped_chain_tree::<S>(
            Size::new(px(60.0), px(50.0)),
            depth,
            |node, leaf| match node {
                2 => NodeInputOf {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    size: Size::new(px(60.0), px(50.0)),
                    align_items: Some(AlignItems::Baseline),
                    ..NodeInputOf::default()
                },
                _ => NodeInputOf {
                    display: if node == leaf {
                        Display::Block
                    } else {
                        Display::Flex
                    },
                    size: Size::new(px(30.0), px(if node == leaf { 20.0 } else { 40.0 })),
                    min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
                    align_items: Some(AlignItems::Start),
                    ..NodeInputOf::default()
                },
            },
        );
    tree = tree.children(2, [3, sibling]).style(
        sibling,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(px(30.0), px(30.0)),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::AUTO),
            ..NodeInputOf::default()
        },
    );
    let nodes = depth + 4;
    let batch = scoped_layout(&tree, nodes);
    let geometry = boxes(&batch, nodes);
    assert_eq!(
        geometry[2].size,
        Size::new(S::from_f64(60.0), S::from_f64(50.0))
    );
    assert_eq!(geometry[2].location, Point::ZERO);
    // The sibling's 30px baseline is the larger ascent, so it sits at the
    // line start beside the first item.
    assert_eq!(
        geometry[sibling as usize].size,
        Size::new(S::from_f64(30.0), S::from_f64(30.0))
    );
    assert_eq!(
        geometry[sibling as usize].location,
        Point::new(S::from_f64(30.0), S::ZERO)
    );
    // The first item's 20px baseline aligns with the 30px line baseline.
    assert_eq!(geometry[3].location, Point::new(S::ZERO, S::from_f64(10.0)));
    for node in 3..leaf {
        assert_eq!(
            geometry[node as usize].size,
            Size::new(S::from_f64(30.0), S::from_f64(40.0)),
            "node {node}"
        );
        if node > 3 {
            assert_eq!(geometry[node as usize].location, Point::ZERO, "node {node}");
        }
    }
    assert_eq!(
        geometry[leaf as usize].size,
        Size::new(S::from_f64(30.0), S::from_f64(20.0))
    );
    assert_eq!(geometry[leaf as usize].location, Point::ZERO);
}

#[test]
fn shallow_scoped_flex_chain_aligns_items_by_propagated_baseline() {
    on_bounded_stack(
        "engine::depth_tests::scoped_flex::shallow_scoped_flex_chain_aligns_items_by_propagated_baseline",
        || {
            assert_scoped_baseline_alignment_chain::<f32>(2);
            assert_scoped_baseline_alignment_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_scoped_flex_chain_aligns_items_by_propagated_baseline() {
    on_bounded_stack(
        "engine::depth_tests::scoped_flex::deep_scoped_flex_chain_aligns_items_by_propagated_baseline",
        || {
            assert_scoped_baseline_alignment_chain::<f32>(24);
            assert_scoped_baseline_alignment_chain::<f64>(24);
        },
    );
}
