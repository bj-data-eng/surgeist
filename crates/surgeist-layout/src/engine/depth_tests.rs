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

std::thread_local! {
    static LEAF_MEASUREMENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn measure_nested_leaf<S: LayoutScalar>(_: crate::LeafMeasureInputOf<S>) -> Size<S> {
    LEAF_MEASUREMENTS.set(LEAF_MEASUREMENTS.get() + 1);
    Size::new(S::from_f64(11.0), S::from_f64(7.0))
}

fn nested_leaf_measurement_count<S: LayoutScalar>(
    display: Display,
    depth: u32,
    bound: bool,
) -> usize {
    LEAF_MEASUREMENTS.set(0);
    let tree = chain::<S>(display, depth + 1)
        .style(
            depth,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .measure_with(depth, measure_nested_leaf);
    let request = standalone();
    let batch = if bound {
        let bound =
            crate::LayoutBoundRequestOf::continuous(crate::LayoutSourceBasis::fresh(), request);
        let target = bound.target_basis();
        let flow = crate::LayoutFlowInput::try_new_bound(
            &tree,
            0,
            bound,
            Vec::new(),
            crate::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch =
            crate::compute_layout_with_flow(&flow, 0, request).expect("bound continuous layout");
        assert_eq!(batch.target_basis(), Some(&target));
        assert!(batch.cache_store_entries().is_empty());
        batch
    } else {
        compute_layout(&tree, 0, request).expect("nested layout")
    };
    assert_eq!(batch.node_layouts().len(), (depth + 1) as usize);
    LEAF_MEASUREMENTS.get()
}

fn assert_nested_leaf_work_is_bounded(display: Display, bound: bool) {
    // This isolates measurement repetition from the separate traversal-depth limit.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let counts = [
                [1, 2, 3, 4].map(|depth| nested_leaf_measurement_count::<f32>(display, depth, bound)),
                [1, 2, 3, 4].map(|depth| nested_leaf_measurement_count::<f64>(display, depth, bound)),
            ];
            assert!(
                counts.iter().flatten().all(|&count| (1..=20).contains(&count)),
                "{display:?} leaf measurements at depths 1–4 must be bounded in both scalar lanes: {counts:?}"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn nested_flex_leaf_measurement_work_is_bounded_per_call() {
    assert_nested_leaf_work_is_bounded(Display::Flex, false);
}

#[test]
fn nested_grid_leaf_measurement_work_is_bounded_per_call() {
    assert_nested_leaf_work_is_bounded(Display::Grid, false);
}

#[test]
fn continuous_bound_flex_leaf_measurement_work_is_bounded_per_call() {
    assert_nested_leaf_work_is_bounded(Display::Flex, true);
}

#[test]
fn continuous_bound_grid_leaf_measurement_work_is_bounded_per_call() {
    assert_nested_leaf_work_is_bounded(Display::Grid, true);
}

fn assert_block_float_trial_chain<S: LayoutScalar>(depth: u32) {
    let px = S::from_f64;
    let preferred = |value| crate::PreferredSizeOf::px(px(value));
    let first_float = depth + 1;
    let mut tree = chain::<S>(Display::Block, depth + 1);
    for level in 0..depth {
        let float = first_float + level;
        tree = tree.children(level, [float, level + 1]).style(
            float,
            NodeInputOf {
                display: Display::Block,
                float: crate::Float::Left,
                size: Size::new(preferred(1.0), preferred(4.0)),
                ..NodeInputOf::default()
            },
        );
        if level != 0 {
            tree = tree.style(
                level,
                NodeInputOf {
                    display: Display::Block,
                    block_flow_context: crate::BlockFlowContext::Independent,
                    ..NodeInputOf::default()
                },
            );
        }
    }
    tree = tree.style(
        depth,
        NodeInputOf {
            display: Display::Block,
            block_flow_context: crate::BlockFlowContext::Independent,
            size: Size::new(crate::PreferredSizeOf::AUTO, preferred(5.0)),
            ..NodeInputOf::default()
        },
    );
    let batch = compute_layout(&tree, 0, standalone()).expect("nested Block float trials");
    assert_eq!(batch.node_layouts().len(), (2 * depth + 1) as usize);
    for node in 0..=depth {
        let output = batch
            .node_layout(node)
            .unwrap()
            .single_box()
            .unwrap()
            .final_output();
        assert_eq!(
            output.size,
            Size::new(px(100.0 - f64::from(node)), px(5.0),),
            "node {node}"
        );
        assert_eq!(
            output.location,
            crate::Point::new(px(f64::from(node != 0)), S::ZERO),
            "node {node}"
        );
        assert_eq!(
            output.source_index,
            crate::SourceIndex::new(usize::from(node != 0)),
            "node {node}"
        );
    }
    for level in 0..depth {
        let float = first_float + level;
        let output = batch
            .node_layout(float)
            .unwrap()
            .single_box()
            .unwrap()
            .final_output();
        assert_eq!(
            output.location,
            crate::Point::new(S::ZERO, S::ZERO),
            "float {float}"
        );
        assert_eq!(output.size, Size::new(px(1.0), px(4.0)), "float {float}");
        assert_eq!(
            output.source_index,
            crate::SourceIndex::new(0),
            "float {float}"
        );
    }
}

#[test]
fn shallow_block_float_trial_layout_publishes_each_bfc() {
    on_bounded_stack(
        "engine::depth_tests::shallow_block_float_trial_layout_publishes_each_bfc",
        || {
            assert_block_float_trial_chain::<f32>(3);
            assert_block_float_trial_chain::<f64>(3);
        },
    );
}

#[test]
fn deep_block_float_trial_layout_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_block_float_trial_layout_returns_on_bounded_stack",
        || {
            assert_block_float_trial_chain::<f32>(64);
            assert_block_float_trial_chain::<f64>(64);
        },
    );
}

fn assert_captured_detached_float_chain<S: LayoutScalar>(depth: u32) {
    use crate::test_support::text::{PreparedParagraph, ahem_style};

    let px = S::from_f64;
    let mut tree = PublicLayoutTreeOf::<S>::new();
    let mut paragraphs = Vec::new();
    for node in 0..=depth {
        let text = depth + 1 + node;
        let paragraph =
            PreparedParagraph::plain(text, surgeist_text::Source::new("X"), ahem_style(5.0, 5.0));
        let mut binding = paragraph.binding(node);
        let children = if node < depth {
            binding.nodes = vec![text, node + 1];
            vec![text, node + 1]
        } else {
            vec![text]
        };
        tree = tree
            .children(node, children)
            .style(
                node,
                NodeInputOf {
                    display: Display::Block,
                    float: if node == 0 {
                        crate::Float::None
                    } else {
                        crate::Float::Left
                    },
                    size: Size::new(
                        crate::PreferredSizeOf::px(px(100.0)),
                        crate::PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .input(text, paragraph.text_input(text));
        paragraphs.push(binding);
    }
    let flow = crate::LayoutFlowInput::try_new(
        &tree,
        0,
        paragraphs,
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .expect("captured Float chain");
    let batch =
        crate::compute_layout_with_flow(&flow, 0, standalone()).expect("captured Float layout");
    assert_eq!(batch.placed_text().len(), (depth + 1) as usize);
    assert_eq!(batch.node_layouts().len(), (2 * (depth + 1)) as usize);
    for node in 0..=depth {
        let output = batch
            .node_layout(node)
            .unwrap()
            .single_box()
            .unwrap()
            .final_output();
        assert_eq!(
            output.size,
            Size::new(px(100.0), px(5.0 * f64::from(depth - node + 1))),
            "node {node}"
        );
        assert_eq!(
            output.location,
            crate::Point::new(S::ZERO, px(if node == 0 { 0.0 } else { 5.0 })),
            "node {node}"
        );
        assert_eq!(
            output.source_index,
            crate::SourceIndex::new(usize::from(node != 0)),
            "node {node}"
        );
        let placed = batch
            .placed_text()
            .iter()
            .find(|placed| placed.container() == node)
            .unwrap();
        assert_eq!(placed.layout().source().text().code_units(), [88]);
        assert_eq!(placed.layout().lines().len(), 1);
    }
}

#[test]
fn shallow_captured_detached_float_chain_publishes_each_paragraph() {
    on_bounded_stack(
        "engine::depth_tests::shallow_captured_detached_float_chain_publishes_each_paragraph",
        || {
            assert_captured_detached_float_chain::<f32>(2);
            assert_captured_detached_float_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_captured_detached_float_chain_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_captured_detached_float_chain_returns_on_bounded_stack",
        || {
            assert_captured_detached_float_chain::<f32>(24);
            assert_captured_detached_float_chain::<f64>(24);
        },
    );
}

fn assert_layout_chain<S: LayoutScalar>(display: Display, position: crate::Position, depth: u32) {
    assert_layout_chain_with_style::<S>(display, position, depth, |_, _, _| {});
}

fn assert_layout_chain_with_style<S: LayoutScalar>(
    display: Display,
    position: crate::Position,
    depth: u32,
    set_style: impl Fn(u32, u32, &mut NodeInputOf<S>),
) {
    assert_layout_chain_with_request(display, position, depth, set_style, standalone());
}

fn assert_layout_chain_with_request<S: LayoutScalar>(
    display: Display,
    position: crate::Position,
    depth: u32,
    set_style: impl Fn(u32, u32, &mut NodeInputOf<S>),
    request: LayoutRootRequestOf<S>,
) {
    let px = |value| crate::PreferredSizeOf::px(S::from_f64(value));
    let inset = |value| {
        crate::LengthAutoOf::value(crate::LengthPercentageOf::px(S::from_f64(value)).unwrap())
    };
    let mut tree = PublicLayoutTreeOf::<S>::new();
    for node in 0..=depth {
        let mut style = NodeInputOf {
            display,
            size: Size::new(px(60.0), px(40.0)),
            ..NodeInputOf::default()
        };
        set_style(node, depth, &mut style);
        if node != 0 && position == crate::Position::Absolute {
            style.position = position;
            style.inset.left = inset(2.0);
            style.inset.top = inset(3.0);
        }
        tree = tree.style(node, style);
        if node < depth {
            let sibling = depth + 1 + node;
            tree = tree.children(node, [node + 1, sibling]).style(
                sibling,
                NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                },
            );
        }
    }
    let batch = compute_layout(&tree, 0, request).expect("layout chain returns");
    assert_eq!(batch.node_layouts().len(), (2 * depth + 1) as usize);
    if position != crate::Position::Absolute {
        assert!(
            batch
                .cache_clear_entries()
                .iter()
                .map(|entry| entry.node())
                .eq((depth + 1..=2 * depth).rev()),
            "hidden cache clears follow first encounter order"
        );
        assert!(
            batch
                .cache_store_entries()
                .iter()
                .all(|entry| entry.node() <= depth)
        );
    }
    for (node, view) in batch.node_layouts() {
        if node > depth {
            let hidden = view.non_box().expect("hidden sibling");
            assert_eq!(hidden.kind(), crate::NonBoxKind::Hidden);
            assert_eq!(hidden.source_index(), crate::SourceIndex::new(1));
            continue;
        }
        let output = view
            .single_box()
            .expect("visible box")
            .final_entry()
            .output();
        assert_eq!(output.size, Size::new(S::from_f64(60.0), S::from_f64(40.0)));
        let location = if node == 0 || position != crate::Position::Absolute {
            crate::Point::ZERO
        } else {
            crate::Point::new(S::from_f64(2.0), S::from_f64(3.0))
        };
        assert_eq!(output.location, location);
        assert_eq!(output.source_index, crate::SourceIndex::new(0));
    }
}

#[test]
fn shallow_positioned_flex_chain_publishes_boxes_and_hidden_siblings() {
    on_bounded_stack(
        "engine::depth_tests::shallow_positioned_flex_chain_publishes_boxes_and_hidden_siblings",
        || {
            assert_layout_chain::<f32>(Display::Flex, crate::Position::Absolute, 2);
            assert_layout_chain::<f64>(Display::Flex, crate::Position::Absolute, 2);
        },
    );
}

#[test]
fn deep_positioned_flex_chain_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_positioned_flex_chain_returns_on_bounded_stack",
        || {
            assert_layout_chain::<f32>(Display::Flex, crate::Position::Absolute, 24);
            assert_layout_chain::<f64>(Display::Flex, crate::Position::Absolute, 24);
        },
    );
}

#[test]
fn shallow_positioned_grid_chain_publishes_boxes_and_hidden_siblings() {
    on_bounded_stack(
        "engine::depth_tests::shallow_positioned_grid_chain_publishes_boxes_and_hidden_siblings",
        || {
            for display in [Display::Grid, Display::GridLanes] {
                assert_layout_chain::<f32>(display, crate::Position::Absolute, 2);
                assert_layout_chain::<f64>(display, crate::Position::Absolute, 2);
            }
        },
    );
}

#[test]
fn deep_positioned_grid_chain_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_positioned_grid_chain_returns_on_bounded_stack",
        || {
            for display in [Display::Grid, Display::GridLanes] {
                assert_layout_chain::<f32>(display, crate::Position::Absolute, 24);
                assert_layout_chain::<f64>(display, crate::Position::Absolute, 24);
            }
        },
    );
}

#[test]
fn shallow_in_flow_flex_chain_publishes_boxes_and_hidden_siblings() {
    on_bounded_stack(
        "engine::depth_tests::shallow_in_flow_flex_chain_publishes_boxes_and_hidden_siblings",
        || {
            assert_layout_chain::<f32>(Display::Flex, crate::Position::Relative, 2);
            assert_layout_chain::<f64>(Display::Flex, crate::Position::Relative, 2);
        },
    );
}

#[test]
fn deep_in_flow_flex_chain_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_in_flow_flex_chain_returns_on_bounded_stack",
        || {
            assert_layout_chain::<f32>(Display::Flex, crate::Position::Relative, 24);
            assert_layout_chain::<f64>(Display::Flex, crate::Position::Relative, 24);
        },
    );
}

#[test]
fn flex_containment_resolves_an_empty_intrinsic_ratio_pass() {
    fn check<S: LayoutScalar>() {
        let tree = PublicLayoutTreeOf::<S>::new().style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    crate::PreferredSizeOf::MAX_CONTENT,
                    crate::PreferredSizeOf::AUTO,
                ),
                min_size: Size::new(
                    crate::MinSizeOf::AUTO,
                    crate::MinSizeOf::value(crate::LengthPercentageOf::px(S::ZERO).unwrap()),
                ),
                size_containment: crate::SizeContainment::BothAxes,
                aspect_ratio: Some(crate::AspectRatioOf::new(S::from_f64(2.0)).unwrap()),
                ..NodeInputOf::default()
            },
        );
        let batch = compute_layout(&tree, 0, standalone()).expect("contained Flex layout returns");
        let output = batch.node_layouts().next().unwrap().1.single_box().unwrap();
        assert_eq!(output.final_entry().output().size, Size::ZERO);
    }

    check::<f32>();
    check::<f64>();
}

#[test]
fn shallow_in_flow_grid_lanes_chain_publishes_boxes_and_hidden_siblings() {
    on_bounded_stack(
        "engine::depth_tests::shallow_in_flow_grid_lanes_chain_publishes_boxes_and_hidden_siblings",
        || {
            assert_layout_chain::<f32>(Display::GridLanes, crate::Position::Relative, 2);
            assert_layout_chain::<f64>(Display::GridLanes, crate::Position::Relative, 2);
        },
    );
}

#[test]
fn deep_in_flow_grid_lanes_chain_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_in_flow_grid_lanes_chain_returns_on_bounded_stack",
        || {
            assert_layout_chain::<f32>(Display::GridLanes, crate::Position::Relative, 24);
            assert_layout_chain::<f64>(Display::GridLanes, crate::Position::Relative, 24);
        },
    );
}

#[test]
fn shallow_in_flow_grid_chain_publishes_boxes_and_hidden_siblings() {
    on_bounded_stack(
        "engine::depth_tests::shallow_in_flow_grid_chain_publishes_boxes_and_hidden_siblings",
        || {
            assert_layout_chain::<f32>(Display::Grid, crate::Position::Relative, 2);
            assert_layout_chain::<f64>(Display::Grid, crate::Position::Relative, 2);
        },
    );
}

#[test]
fn deep_in_flow_grid_chain_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_in_flow_grid_chain_returns_on_bounded_stack",
        || {
            assert_layout_chain::<f32>(Display::Grid, crate::Position::Relative, 24);
            assert_layout_chain::<f64>(Display::Grid, crate::Position::Relative, 24);
        },
    );
}

fn assert_intrinsic_grid_chain<S: LayoutScalar>(display: Display, depth: u32) {
    assert_layout_chain_with_style::<S>(
        display,
        crate::Position::Relative,
        depth,
        |node, leaf, style| {
            if node != leaf {
                style.size.height = crate::PreferredSizeOf::MAX_CONTENT;
            }
        },
    );
}

#[test]
fn shallow_intrinsic_grid_chain_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::shallow_intrinsic_grid_chain_uses_leaf_height",
        || {
            assert_intrinsic_grid_chain::<f32>(Display::Grid, 2);
            assert_intrinsic_grid_chain::<f64>(Display::Grid, 2);
        },
    );
}

#[test]
fn deep_intrinsic_grid_chain_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::deep_intrinsic_grid_chain_uses_leaf_height",
        || {
            assert_intrinsic_grid_chain::<f32>(Display::Grid, 24);
            assert_intrinsic_grid_chain::<f64>(Display::Grid, 24);
        },
    );
}

#[test]
fn shallow_intrinsic_grid_lanes_chain_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::shallow_intrinsic_grid_lanes_chain_uses_leaf_height",
        || {
            assert_intrinsic_grid_chain::<f32>(Display::GridLanes, 2);
            assert_intrinsic_grid_chain::<f64>(Display::GridLanes, 2);
        },
    );
}

#[test]
fn deep_intrinsic_grid_lanes_chain_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::deep_intrinsic_grid_lanes_chain_uses_leaf_height",
        || {
            assert_intrinsic_grid_chain::<f32>(Display::GridLanes, 24);
            assert_intrinsic_grid_chain::<f64>(Display::GridLanes, 24);
        },
    );
}

fn assert_intrinsic_grid_lanes_auto_repeat_chain<S: LayoutScalar>(depth: u32) {
    assert_layout_chain_with_style::<S>(
        Display::GridLanes,
        crate::Position::Relative,
        depth,
        |node, leaf, style| {
            if node != leaf {
                style.size.height = crate::PreferredSizeOf::MAX_CONTENT;
                style.grid_template_columns = vec![crate::TrackComponentOf::Repeat(
                    crate::TrackRepetitionOf::auto_fill_components(vec![
                        crate::TrackComponentOf::MAX_CONTENT,
                    ])
                    .unwrap(),
                )];
            }
        },
    );
}

#[test]
fn shallow_intrinsic_grid_lanes_auto_repeat_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::shallow_intrinsic_grid_lanes_auto_repeat_uses_leaf_height",
        || {
            assert_intrinsic_grid_lanes_auto_repeat_chain::<f32>(2);
            assert_intrinsic_grid_lanes_auto_repeat_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_intrinsic_grid_lanes_auto_repeat_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::deep_intrinsic_grid_lanes_auto_repeat_uses_leaf_height",
        || {
            assert_intrinsic_grid_lanes_auto_repeat_chain::<f32>(24);
            assert_intrinsic_grid_lanes_auto_repeat_chain::<f64>(24);
        },
    );
}

fn assert_auto_height_grid_lanes_chain<S: LayoutScalar>(depth: u32) {
    assert_layout_chain_with_style::<S>(
        Display::GridLanes,
        crate::Position::Relative,
        depth,
        |node, leaf, style| {
            if node != leaf {
                style.size.height = crate::PreferredSizeOf::AUTO;
                style.grid_template_columns = vec![crate::TrackComponentOf::from(
                    crate::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                )];
            }
        },
    );
}

#[test]
fn shallow_auto_height_grid_lanes_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::shallow_auto_height_grid_lanes_uses_leaf_height",
        || {
            assert_auto_height_grid_lanes_chain::<f32>(2);
            assert_auto_height_grid_lanes_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_auto_height_grid_lanes_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::deep_auto_height_grid_lanes_uses_leaf_height",
        || {
            assert_auto_height_grid_lanes_chain::<f32>(24);
            assert_auto_height_grid_lanes_chain::<f64>(24);
        },
    );
}

fn assert_accepted_grid_sizing_chain<S: LayoutScalar>(depth: u32) {
    let mut tree = PublicLayoutTreeOf::<S>::new();
    let track =
        crate::TrackComponentOf::from(crate::LengthPercentageOf::px(S::from_f64(30.0)).unwrap());
    for node in 0..=depth {
        let mut style = NodeInputOf {
            display: Display::Grid,
            size: Size::new(
                crate::PreferredSizeOf::AUTO,
                crate::PreferredSizeOf::px(S::from_f64(40.0)),
            ),
            grid_template_columns: vec![track.clone(), track.clone()],
            grid_template_rows: vec![crate::TrackComponentOf::from(
                crate::LengthPercentageOf::px(S::from_f64(40.0)).unwrap(),
            )],
            justify_items: Some(crate::AlignItems::Start),
            align_items: Some(crate::AlignItems::Baseline),
            grid_column: crate::GridPlacement::try_span(2).unwrap(),
            ..NodeInputOf::default()
        };
        style.min_size.width = crate::MinSizeOf::value(
            crate::LengthPercentageOf::from_percent_fraction(S::ONE).unwrap(),
        );
        style.gap.width = crate::LengthOf::value(
            crate::LengthPercentageOf::from_percent_fraction(S::from_f64(0.75)).unwrap(),
        );
        tree = tree.style(node, style);
        if node < depth {
            tree = tree.children(node, [node + 1, depth + 1 + node]).style(
                depth + 1 + node,
                NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                },
            );
        }
    }
    let sizing = SizingRequestOf::new(
        Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Indefinite,
            SizingOperation::Normal,
        )),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let request = LayoutRootRequestOf::standalone(
        sizing,
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    );
    let batch = compute_layout(&tree, 0, request).expect("accepted sizing chain returns");
    assert_eq!(batch.node_layouts().len(), (2 * depth + 1) as usize);
    // The initial two-track width is 60. Each child spans those tracks and a
    // gap equal to three quarters of its parent's unrounded width.
    let mut widths = vec![S::from_f64(60.0)];
    for index in 0..depth as usize {
        widths.push(S::from_f64(60.0) + S::from_f64(0.75) * widths[index]);
    }
    for (node, view) in batch.node_layouts() {
        if node > depth {
            let hidden = view.non_box().expect("hidden sibling");
            assert_eq!(hidden.kind(), crate::NonBoxKind::Hidden);
            assert_eq!(hidden.source_index(), crate::SourceIndex::new(1));
            continue;
        }
        let output = view
            .single_box()
            .expect("visible grid")
            .final_entry()
            .output();
        assert_eq!(
            output.size,
            Size::new(widths[node as usize].round(), S::from_f64(40.0))
        );
        assert_eq!(output.location, crate::Point::ZERO);
        assert_eq!(output.source_index, crate::SourceIndex::new(0));
    }
}

#[test]
fn shallow_accepted_grid_sizing_preserves_percentage_gaps() {
    on_bounded_stack(
        "engine::depth_tests::shallow_accepted_grid_sizing_preserves_percentage_gaps",
        || {
            assert_accepted_grid_sizing_chain::<f32>(2);
            assert_accepted_grid_sizing_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_accepted_grid_sizing_preserves_percentage_gaps() {
    on_bounded_stack(
        "engine::depth_tests::deep_accepted_grid_sizing_preserves_percentage_gaps",
        || {
            assert_accepted_grid_sizing_chain::<f32>(24);
            assert_accepted_grid_sizing_chain::<f64>(24);
        },
    );
}

fn assert_shrinking_flex_chain<S: LayoutScalar>(depth: u32) {
    assert_layout_chain_with_style::<S>(
        Display::Flex,
        crate::Position::Relative,
        depth,
        |node, leaf, style| {
            style.size.width =
                crate::PreferredSizeOf::px(S::from_f64(if node == 0 { 60.0 } else { 120.0 }));
            style.min_size.width =
                crate::MinSizeOf::value(crate::LengthPercentageOf::px(S::ZERO).unwrap());
            style.align_items = Some(crate::AlignItems::Start);
            if node != leaf {
                style.size.height = crate::PreferredSizeOf::AUTO;
            }
        },
    );
}

#[test]
fn shallow_shrinking_flex_chain_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::shallow_shrinking_flex_chain_uses_leaf_height",
        || {
            assert_shrinking_flex_chain::<f32>(2);
            assert_shrinking_flex_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_shrinking_flex_chain_uses_leaf_height() {
    on_bounded_stack(
        "engine::depth_tests::deep_shrinking_flex_chain_uses_leaf_height",
        || {
            assert_shrinking_flex_chain::<f32>(24);
            assert_shrinking_flex_chain::<f64>(24);
        },
    );
}

fn assert_recollected_flex_chain<S: LayoutScalar>(depth: u32) {
    let sizing = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let request = LayoutRootRequestOf::standalone(
        sizing,
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    );
    assert_layout_chain_with_request::<S>(
        Display::Flex,
        crate::Position::Relative,
        depth,
        |node, leaf, style| {
            style.flex_direction = crate::FlexDirection::Column;
            style.flex_wrap = crate::FlexWrap::Wrap;
            style.align_items = Some(crate::AlignItems::Start);
            style.min_size.height =
                crate::MinSizeOf::value(crate::LengthPercentageOf::px(S::ZERO).unwrap());
            if node != leaf {
                style.size.width = crate::PreferredSizeOf::AUTO;
            }
        },
        request,
    );
}

#[test]
fn shallow_recollected_flex_chain_uses_leaf_width() {
    on_bounded_stack(
        "engine::depth_tests::shallow_recollected_flex_chain_uses_leaf_width",
        || {
            assert_recollected_flex_chain::<f32>(2);
            assert_recollected_flex_chain::<f64>(2);
        },
    );
}

#[test]
fn deep_recollected_flex_chain_uses_leaf_width() {
    on_bounded_stack(
        "engine::depth_tests::deep_recollected_flex_chain_uses_leaf_width",
        || {
            assert_recollected_flex_chain::<f32>(24);
            assert_recollected_flex_chain::<f64>(24);
        },
    );
}
