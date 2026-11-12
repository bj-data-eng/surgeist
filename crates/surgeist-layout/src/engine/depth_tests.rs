use std::process::Command;

use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    AvailableSpaceOf, AxisSizingRequestOf, Direction, Display, FlowAxes, LayoutRootRequestOf,
    LayoutScalar, NearestScrollportOf, NodeInputOf, NonNegativeFiniteOf, OrthogonalFallbackLimitOf,
    OrthogonalFallbackOf, Size, SizingOperation, SizingRequestOf, WritingMode, compute_layout,
};

fn chain<S: LayoutScalar>(display: Display, nodes: u32) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new();
    for node in 0..nodes {
        tree = tree.style(
            node,
            NodeInputOf {
                display,
                ..NodeInputOf::default()
            },
        );
        if node + 1 < nodes {
            tree.insert_children(node, [node + 1]);
        }
    }
    tree
}

fn standalone<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    let sizing = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    LayoutRootRequestOf::standalone(
        sizing,
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    )
}

fn on_bounded_stack(test: &str, run: impl FnOnce() + Send + 'static) {
    const CHILD: &str = "SURGEIST_TEST_LAYOUT_TREE_DEPTH_CHILD";
    const STACK_BYTES: usize = 2 * 1024 * 1024;

    if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new(test)) {
        std::thread::Builder::new()
            .name("bounded-layout-stack".into())
            .stack_size(STACK_BYTES)
            .spawn(run)
            .unwrap()
            .join()
            .unwrap();
        return;
    }

    let child = Command::new(std::env::current_exe().unwrap())
        .args([test, "--exact", "--nocapture"])
        .env(CHILD, test)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&child.stdout);
    let stderr = String::from_utf8_lossy(&child.stderr);
    assert!(
        child.status.success() && stdout.contains("1 passed"),
        "{test} must return on a {STACK_BYTES}-byte stack, child {}\n{stdout}\n{stderr}",
        child.status,
    );
}

fn assert_hidden_layout<S: LayoutScalar>(
    tree: PublicLayoutTreeOf<S>,
    nodes: u32,
    source_index: impl Fn(u32) -> usize,
) {
    let tree = tree.style(
        0,
        NodeInputOf {
            display: Display::None,
            ..NodeInputOf::default()
        },
    );
    let batch = compute_layout(&tree, 0, standalone()).expect("hidden layout must return");
    assert_eq!(batch.node_layouts().len(), nodes as usize);
    assert!(
        batch
            .cache_clear_entries()
            .iter()
            .map(|entry| entry.node())
            .eq(0..nodes),
        "cache clears follow source order"
    );
    assert!(batch.cache_store_entries().is_empty());
    assert!(batch.fragments().next().is_none());
    for (node, view) in batch.node_layouts() {
        let hidden = view.non_box().expect("hidden publication");
        assert_eq!(
            hidden.kind(),
            crate::NonBoxKind::Hidden,
            "hidden node {node}"
        );
        assert_eq!(
            hidden.final_entry().output().size,
            Size::ZERO,
            "hidden node {node}"
        );
        assert_eq!(
            hidden.source_index(),
            crate::SourceIndex::new(source_index(node)),
            "hidden node {node}"
        );
    }
}

fn assert_hidden_sibling_order<S: LayoutScalar>() {
    let tree = chain::<S>(Display::Block, 4)
        .children(0, [1, 3])
        .children(2, []);
    assert_hidden_layout(tree, 4, |node| usize::from(node == 3));
}

#[test]
fn deep_hidden_layout_clears_descendants_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_hidden_layout_clears_descendants_on_bounded_stack",
        || {
            assert_hidden_sibling_order::<f32>();
            assert_hidden_sibling_order::<f64>();
            assert_hidden_layout(chain::<f32>(Display::Block, 4096), 4096, |_| 0);
            assert_hidden_layout(chain::<f64>(Display::Block, 4096), 4096, |_| 0);
        },
    );
}

fn assert_block_chain<S: LayoutScalar>(depth: u32) {
    let block = |height| NodeInputOf {
        display: Display::Block,
        size: Size::new(
            crate::PreferredSizeOf::AUTO,
            crate::PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..NodeInputOf::default()
    };
    let tree = chain::<S>(Display::Block, depth + 1)
        .children(0, [1, depth])
        .children(depth - 1, [])
        .style(depth - 1, block(12.0))
        .style(depth, block(7.0));
    let batch = compute_layout(&tree, 0, standalone()).expect("Block chain must lay out");
    assert_eq!(batch.node_layouts().len(), (depth + 1) as usize);
    for (node, view) in batch.node_layouts() {
        let output = view
            .single_box()
            .expect("Block box must publish")
            .final_output();
        let (height, top, source_index) = match node {
            0 => (19.0, 0.0, 0),
            node if node == depth => (7.0, 12.0, 1),
            _ => (12.0, 0.0, 0),
        };
        assert_eq!(
            output.size,
            Size::new(S::from_f64(100.0), S::from_f64(height)),
            "node {node}"
        );
        assert_eq!(
            output.location,
            crate::Point::new(S::ZERO, S::from_f64(top)),
            "node {node}"
        );
        assert_eq!(
            output.source_index,
            crate::SourceIndex::new(source_index),
            "node {node}"
        );
    }
}

#[test]
fn deep_block_layout_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_block_layout_returns_on_bounded_stack",
        || {
            assert_block_chain::<f32>(3);
            assert_block_chain::<f64>(3);
            assert_block_chain::<f32>(1024);
            assert_block_chain::<f64>(1024);
        },
    );
}

fn assert_scalar_block_chain<S: LayoutScalar>(nodes: u32) {
    let width = S::from_f64(13.0);
    let tree =
        chain::<S>(Display::Block, nodes).measure(nodes - 1, Size::new(width, S::from_f64(7.0)));
    let mut session = super::session::ComputeSession::new(&tree, Vec::new());
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let input = crate::ComputeInputOf::for_child(
        crate::RunMode::ComputeSize,
        crate::SizingMode::ContentSize,
        crate::RequestedAxis::Horizontal,
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(axes, crate::ParentFormattingContext::NoParent),
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
    );
    let measured = crate::Compute::compute_axis_demand(
        &mut session,
        0,
        input,
        crate::PhysicalAxis::Horizontal,
    )
    .expect("scalar Block demand must return");
    assert_eq!(measured, width);
}

#[test]
fn deep_block_scalar_inline_demand_returns() {
    on_bounded_stack(
        "engine::depth_tests::deep_block_scalar_inline_demand_returns",
        || {
            assert_scalar_block_chain::<f32>(3);
            assert_scalar_block_chain::<f64>(3);
            assert_scalar_block_chain::<f32>(1024);
            assert_scalar_block_chain::<f64>(1024);
        },
    );
}

fn assert_intrinsic_block_chain<S: LayoutScalar>(nodes: u32) {
    let expected = Size::new(S::from_f64(13.0), S::from_f64(7.0));
    let tree = chain::<S>(Display::Block, nodes).measure(nodes - 1, expected);
    let request = LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    );
    let batch = compute_layout(&tree, 0, request).expect("intrinsically sized Block chain");
    assert_eq!(batch.node_layouts().len(), nodes as usize);
    for (node, view) in batch.node_layouts() {
        let output = view.single_box().unwrap().final_output();
        assert_eq!(output.size, expected, "node {node}");
        assert_eq!(output.location, crate::Point::new(S::ZERO, S::ZERO));
        assert_eq!(output.source_index, crate::SourceIndex::ZERO);
    }
}

#[test]
fn shallow_intrinsic_block_layout_publishes_geometry() {
    on_bounded_stack(
        "engine::depth_tests::shallow_intrinsic_block_layout_publishes_geometry",
        || {
            assert_intrinsic_block_chain::<f32>(3);
            assert_intrinsic_block_chain::<f64>(3);
        },
    );
}

#[test]
fn deep_intrinsic_block_layout_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_intrinsic_block_layout_returns_on_bounded_stack",
        || {
            assert_intrinsic_block_chain::<f32>(128);
            assert_intrinsic_block_chain::<f64>(128);
        },
    );
}

fn assert_positioned_block_chain<S: LayoutScalar>(depth: u32) {
    let (width, height) = (S::from_f64(13.0), S::from_f64(7.0));
    let length = |value| crate::LengthPercentageOf::px(value).unwrap();
    let preferred = |value| crate::PreferredSizeOf::value(length(value));
    let positioned = |left, top| NodeInputOf {
        display: Display::Block,
        position: crate::Position::Absolute,
        size: Size::new(preferred(width), preferred(height)),
        inset: crate::Edges {
            left: crate::LengthAutoOf::value(length(S::from_f64(left))),
            top: crate::LengthAutoOf::value(length(S::from_f64(top))),
            ..crate::Edges::all(crate::LengthAutoOf::AUTO)
        },
        ..NodeInputOf::default()
    };
    let mut tree = chain::<S>(Display::Block, depth + 1)
        .children(0, [1, depth])
        .children(depth - 1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(preferred(S::from_f64(100.0)), preferred(S::from_f64(40.0))),
                ..NodeInputOf::default()
            },
        )
        .style(depth, positioned(25.0, 5.0));
    for node in 1..depth {
        tree = tree.style(node, positioned(2.0, 3.0));
    }

    let batch = compute_layout(&tree, 0, standalone()).expect("positioned Block chain");
    assert_eq!(batch.node_layouts().len(), (depth + 1) as usize);
    for (node, view) in batch.node_layouts() {
        let output = view.single_box().unwrap().final_output();
        let (size, x, y, source_index) = if node == 0 {
            (
                Size::new(S::from_f64(100.0), S::from_f64(40.0)),
                0.0,
                0.0,
                0,
            )
        } else if node == depth {
            (Size::new(width, height), 25.0, 5.0, 1)
        } else {
            (Size::new(width, height), 2.0, 3.0, 0)
        };
        assert_eq!(output.size, size, "node {node}");
        assert_eq!(
            output.location,
            crate::Point::new(S::from_f64(x), S::from_f64(y)),
            "node {node}"
        );
        assert_eq!(
            output.source_index,
            crate::SourceIndex::new(source_index),
            "node {node}"
        );
    }
}

#[test]
fn shallow_positioned_block_layout_publishes_geometry() {
    on_bounded_stack(
        "engine::depth_tests::shallow_positioned_block_layout_publishes_geometry",
        || {
            assert_positioned_block_chain::<f32>(3);
            assert_positioned_block_chain::<f64>(3);
        },
    );
}

#[test]
fn deep_positioned_block_layout_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_positioned_block_layout_returns_on_bounded_stack",
        || {
            assert_positioned_block_chain::<f32>(128);
            assert_positioned_block_chain::<f64>(128);
        },
    );
}

fn assert_shared_float_block_chain<S: LayoutScalar>(depth: u32) {
    let px = S::from_f64;
    let preferred =
        |value| crate::PreferredSizeOf::value(crate::LengthPercentageOf::px(px(value)).unwrap());
    let floated = |width, height| NodeInputOf {
        display: Display::Block,
        float: crate::Float::Left,
        size: Size::new(preferred(width), preferred(height)),
        ..NodeInputOf::default()
    };
    let tree = chain::<S>(Display::Block, depth + 1)
        .children(0, [depth, 1])
        .children(depth - 1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(preferred(100.0), preferred(80.0)),
                ..NodeInputOf::default()
            },
        )
        .style(depth, floated(80.0, 20.0))
        .style(depth - 1, floated(30.0, 10.0));
    let batch = compute_layout(&tree, 0, standalone()).expect("shared float Block chain");
    assert_eq!(batch.node_layouts().len(), (depth + 1) as usize);
    let output = |node| {
        batch
            .node_layout(node)
            .unwrap()
            .single_box()
            .unwrap()
            .final_output()
    };
    assert_eq!(output(depth).location, crate::Point::new(S::ZERO, S::ZERO));
    assert_eq!(output(1).source_index, crate::SourceIndex::new(1));
    assert_eq!(output(depth - 1).size, Size::new(px(30.0), px(10.0)));
    assert_eq!(
        output(depth - 1).location,
        crate::Point::new(S::ZERO, px(20.0))
    );
}

#[test]
fn shallow_shared_float_block_layout_publishes_geometry() {
    on_bounded_stack(
        "engine::depth_tests::shallow_shared_float_block_layout_publishes_geometry",
        || {
            assert_shared_float_block_chain::<f32>(4);
            assert_shared_float_block_chain::<f64>(4);
        },
    );
}

#[test]
fn deep_shared_float_block_layout_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_shared_float_block_layout_returns_on_bounded_stack",
        || {
            assert_shared_float_block_chain::<f32>(128);
            assert_shared_float_block_chain::<f64>(128);
        },
    );
}

fn atomic_inline_block_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let px = S::from_f64;
    let preferred =
        |value| crate::PreferredSizeOf::value(crate::LengthPercentageOf::px(px(value)).unwrap());
    let atomic = || NodeInputOf {
        display: Display::InlineBlock,
        atomic_inline_participation: Some(crate::AtomicInlineParticipation::new(
            surgeist_text::BidiLevel::try_new(0).unwrap(),
            crate::InlineBreakOpportunity::Allowed,
        )),
        inline_baseline_context: crate::InlineBaselineContext::Physical,
        vertical_align: crate::VerticalAlignOf::Top,
        size: Size::new(preferred(13.0), preferred(7.0)),
        ..NodeInputOf::default()
    };
    let mut tree = chain::<S>(Display::Block, depth + 1)
        .children(0, [1, depth])
        .children(depth - 1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                inline_baseline_context: crate::InlineBaselineContext::Physical,
                size: Size::new(preferred(100.0), preferred(80.0)),
                ..NodeInputOf::default()
            },
        )
        .style(depth, atomic());
    for node in 1..depth {
        tree = tree.style(node, atomic());
    }
    tree
}

fn assert_atomic_inline_block_chain<S: LayoutScalar>(depth: u32) {
    let px = S::from_f64;
    let tree = atomic_inline_block_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("atomic inline Block chain");
    assert_eq!(batch.node_layouts().len(), (depth + 1) as usize);
    for (node, view) in batch.node_layouts() {
        let output = view.single_box().unwrap().final_output();
        let (width, height, x, source_index) = match node {
            0 => (100.0, 80.0, 0.0, 0),
            node if node == depth => (13.0, 7.0, 13.0, 1),
            _ => (13.0, 7.0, 0.0, 0),
        };
        assert_eq!(output.size, Size::new(px(width), px(height)), "node {node}");
        assert_eq!(
            output.location,
            crate::Point::new(px(x), S::ZERO),
            "node {node}"
        );
        assert_eq!(
            output.source_index,
            crate::SourceIndex::new(source_index),
            "node {node}"
        );
    }
}

#[test]
fn shallow_atomic_inline_block_layout_publishes_geometry() {
    on_bounded_stack(
        "engine::depth_tests::shallow_atomic_inline_block_layout_publishes_geometry",
        || {
            assert_atomic_inline_block_chain::<f32>(3);
            assert_atomic_inline_block_chain::<f64>(3);
        },
    );
}

#[test]
fn deep_atomic_inline_block_layout_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_atomic_inline_block_layout_returns_on_bounded_stack",
        || {
            assert_atomic_inline_block_chain::<f32>(128);
            assert_atomic_inline_block_chain::<f64>(128);
        },
    );
}

fn assert_atomic_inline_leaf_refusal<S: LayoutScalar>(depth: u32) {
    let leaf = depth + 1;
    let tree = atomic_inline_block_chain::<S>(depth)
        .children(depth - 1, [leaf])
        .style(leaf, NodeInputOf::default())
        .measure(leaf, Size::new(S::NAN, S::from_f64(7.0)));
    let error = compute_layout(&tree, 0, standalone()).unwrap_err();
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(leaf));
    assert_eq!(error.operation(), crate::LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::MeasurementOutput(output))
            if output.axis() == crate::PhysicalAxis::Horizontal
                && matches!(output.error(), crate::NonNegativeFiniteScalarErrorOf::NonFinite { value } if value.to_f64().is_nan())
    ));

    let repaired = tree.measure(leaf, Size::new(S::from_f64(5.0), S::from_f64(7.0)));
    let batch =
        compute_layout(&repaired, 0, standalone()).expect("valid leaf returns a complete batch");
    assert_eq!(batch.node_layouts().len(), (depth + 2) as usize);
}

#[test]
fn deep_atomic_inline_block_layout_returns_the_original_leaf_refusal() {
    on_bounded_stack(
        "engine::depth_tests::deep_atomic_inline_block_layout_returns_the_original_leaf_refusal",
        || {
            assert_atomic_inline_leaf_refusal::<f32>(128);
            assert_atomic_inline_leaf_refusal::<f64>(128);
        },
    );
}
