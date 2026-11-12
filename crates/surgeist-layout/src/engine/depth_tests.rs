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

#[test]
fn deep_block_layout_returns_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::deep_block_layout_returns_on_bounded_stack",
        || {
            let control = chain::<f64>(Display::Block, 3);
            compute_layout(&control, 0, standalone()).expect("shallow control must lay out");
            let deep = chain::<f64>(Display::Block, 1024);
            let _ = compute_layout(&deep, 0, standalone());
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
