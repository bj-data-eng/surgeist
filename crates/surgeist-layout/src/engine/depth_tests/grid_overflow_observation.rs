use super::*;
use crate::{
    geometry::Point,
    measurement::LeafMeasureInputOf,
    node_input::{ComputedOverflow, Overflow},
    sizing::PreferredSizeOf,
};

fn valid_measurement<S: LayoutScalar>(_: LeafMeasureInputOf<S>) -> Size<S> {
    Size::new(S::from_f64(60.0), S::from_f64(40.0))
}

fn refuse_final_placement<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
    let axes = input.content_sizing_request().axes();
    // Performed leaf requests expose no authoritative known dimensions to
    // the provider. Their actual settled content offers carry the final area.
    // This tuple selects a placement request, not a public observer identity.
    if input.known_content_size() == Size::NONE
        && axes.width.operation() == SizingOperation::Normal
        && axes.height.operation() == SizingOperation::Normal
        && axes.width.space().definite_size().map(|value| value.get()) == Some(S::from_f64(60.0))
        && axes.height.space().definite_size().map(|value| value.get()) == Some(S::from_f64(40.0))
    {
        Size::new(S::from_f64(60.0), S::NAN)
    } else {
        valid_measurement(input)
    }
}

fn overflow_grid_chain<S: LayoutScalar>(
    container: Display,
    depth: u32,
    hidden_subtree: bool,
    measure: fn(LeafMeasureInputOf<S>) -> Size<S>,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new();
    for node in 0..=depth {
        let terminal = node == depth;
        tree = tree.style(
            node,
            NodeInputOf {
                display: if terminal { Display::Block } else { container },
                // A Grid Lanes container has one 60px lane; the other display
                // keeps its implicit automatic track.
                grid_template_columns: if !terminal && container == Display::GridLanes {
                    vec![crate::value::TrackComponentOf::from(
                        crate::value::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                    )]
                } else {
                    Vec::new()
                },
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(60.0)),
                    if terminal {
                        PreferredSizeOf::px(S::from_f64(40.0))
                    } else {
                        PreferredSizeOf::MAX_CONTENT
                    },
                ),
                overflow: if terminal {
                    ComputedOverflow::VISIBLE
                } else {
                    ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap()
                },
                scrollbar_width: crate::node_input::ScrollbarWidthOf::try_new(S::from_f64(1.0))
                    .unwrap(),
                ..NodeInputOf::default()
            },
        );
        if !terminal {
            // Preserve the existing intrinsic-chain source-order control.
            tree = tree.children(node, [node + 1, depth + 1 + node]).style(
                depth + 1 + node,
                NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                },
            );
        }
    }
    if hidden_subtree {
        // One supported hidden subtree is reachable from every visible
        // container. Its nonleaf hidden parent excludes completed-request
        // reuse without adding any visible container levels or descendant
        // body layout.
        tree = tree.children(2 * depth, [2 * depth + 1]).style(
            2 * depth + 1,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    }
    tree.measure_with(depth, measure)
}

fn assert_overflow_grid_geometry<S: LayoutScalar>(
    container: Display,
    depth: u32,
    hidden_subtree: bool,
) {
    let tree = overflow_grid_chain::<S>(container, depth, hidden_subtree, valid_measurement::<S>);
    assert_overflow_grid_tree_geometry(&tree, depth, hidden_subtree);
}

fn assert_overflow_grid_tree_geometry<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    depth: u32,
    hidden_subtree: bool,
) {
    let batch = compute_layout(tree, 0, standalone()).expect("automatic-overflow Grid layout");
    assert_overflow_grid_batch_geometry(&batch, depth, hidden_subtree);
}

fn assert_overflow_grid_batch_geometry<S: LayoutScalar>(
    batch: &crate::output::CompletedLayoutBatchOf<u32, S>,
    depth: u32,
    hidden_subtree: bool,
) {
    assert_eq!(
        batch.node_layouts().len(),
        (2 * depth + 1 + u32::from(hidden_subtree)) as usize
    );
    // One visible 60x40 item fills the default automatic track on each axis,
    // or the single 60px lane of a Grid Lanes container whose lane length is
    // the item's 40px block size.
    // Max-content block sizing encloses that item exactly. No edges, margins,
    // gaps or positioned content can require an actual automatic scrollbar.
    for (node, layout) in batch.node_layouts() {
        if node > depth {
            let hidden = layout.non_box().expect("hidden sibling");
            assert_eq!(
                hidden.kind(),
                crate::block_fragmentation::NonBoxKind::Hidden
            );
            assert_eq!(
                hidden.source_index(),
                crate::output::SourceIndex::new(usize::from(node <= 2 * depth))
            );
            assert_eq!(hidden.final_geometry().location, Point::ZERO);
            assert_eq!(hidden.final_geometry().size, Size::ZERO);
        } else {
            let output = layout.single_box().expect("visible box").final_output();
            assert_eq!(
                output.size,
                Size::new(S::from_f64(60.0), S::from_f64(40.0)),
                "node {node}"
            );
            assert_eq!(output.location, Point::ZERO, "node {node}");
            assert_eq!(
                output.source_index,
                crate::output::SourceIndex::ZERO,
                "node {node}"
            );
            assert_eq!(output.scrollbar_size(), Size::ZERO, "node {node}");
        }
    }
    assert!(batch.cache_clear_entries().iter().map(|row| row.node()).eq(
        (depth + 1..=2 * depth).rev().flat_map(|node| {
            // A hidden parent is cleared before its source-zero child;
            // the next ancestor's hidden sibling follows that subtree.
            core::iter::once(node)
                .chain((hidden_subtree && node == 2 * depth).then_some(2 * depth + 1))
        })
    ));
}

fn assert_overflow_grid_refusal<S: LayoutScalar>(
    container: Display,
    depth: u32,
    hidden_subtree: bool,
) {
    let tree = overflow_grid_chain::<S>(
        container,
        depth,
        hidden_subtree,
        refuse_final_placement::<S>,
    );
    let error = compute_layout(&tree, 0, standalone()).expect_err("invalid final leaf output");
    assert_eq!(error.site(), crate::error::LayoutErrorSiteOf::Node(depth));
    assert_eq!(
        error.operation(),
        crate::error::LayoutOperation::LeafMeasurement
    );
    assert!(matches!(
        error.kind(),
        crate::error::LayoutErrorKindOf::InvalidInput(crate::error::LayoutInvalidInputOf::MeasurementOutput(output))
            if output.axis() == crate::geometry::PhysicalAxis::Vertical
                && matches!(output.error(), crate::value::NonNegativeFiniteScalarErrorOf::NonFinite { value } if value.to_f64().is_nan())
    ));
}

#[test]
fn shallow_grid_overflow_observation_preserves_geometry_and_refusal() {
    on_bounded_stack(
        "engine::depth_tests::grid_overflow_observation::shallow_grid_overflow_observation_preserves_geometry_and_refusal",
        || {
            for hidden_subtree in [false, true] {
                assert_overflow_grid_geometry::<f32>(Display::Grid, 2, hidden_subtree);
                assert_overflow_grid_refusal::<f32>(Display::Grid, 2, hidden_subtree);
                assert_overflow_grid_geometry::<f64>(Display::Grid, 2, hidden_subtree);
                assert_overflow_grid_refusal::<f64>(Display::Grid, 2, hidden_subtree);
            }
        },
    );
}

#[test]
fn shallow_grid_lanes_overflow_observation_preserves_geometry_and_refusal() {
    on_bounded_stack(
        "engine::depth_tests::grid_overflow_observation::shallow_grid_lanes_overflow_observation_preserves_geometry_and_refusal",
        || {
            for hidden_subtree in [false, true] {
                assert_overflow_grid_geometry::<f32>(Display::GridLanes, 2, hidden_subtree);
                assert_overflow_grid_refusal::<f32>(Display::GridLanes, 2, hidden_subtree);
                assert_overflow_grid_geometry::<f64>(Display::GridLanes, 2, hidden_subtree);
                assert_overflow_grid_refusal::<f64>(Display::GridLanes, 2, hidden_subtree);
            }
        },
    );
}

macro_rules! deep_grid_observation_test {
    ($name:ident, $display:expr, $scalar:ty, $check:ident, $hidden_subtree:expr) => {
        #[test]
        fn $name() {
            on_bounded_stack(
                concat!(
                    "engine::depth_tests::grid_overflow_observation::",
                    stringify!($name)
                ),
                || $check::<$scalar>($display, 24, $hidden_subtree),
            );
        }
    };
}

deep_grid_observation_test!(
    deep_grid_overflow_observation_preserves_geometry_f32,
    Display::Grid,
    f32,
    assert_overflow_grid_geometry,
    false
);
deep_grid_observation_test!(
    deep_grid_overflow_observation_preserves_geometry_f64,
    Display::Grid,
    f64,
    assert_overflow_grid_geometry,
    false
);
deep_grid_observation_test!(
    deep_grid_overflow_observation_returns_leaf_refusal_f32,
    Display::Grid,
    f32,
    assert_overflow_grid_refusal,
    false
);
deep_grid_observation_test!(
    deep_grid_overflow_observation_returns_leaf_refusal_f64,
    Display::Grid,
    f64,
    assert_overflow_grid_refusal,
    false
);

deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_preserves_geometry_f32,
    Display::Grid,
    f32,
    assert_overflow_grid_geometry,
    true
);
deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_preserves_geometry_f64,
    Display::Grid,
    f64,
    assert_overflow_grid_geometry,
    true
);
deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_returns_leaf_refusal_f32,
    Display::Grid,
    f32,
    assert_overflow_grid_refusal,
    true
);
deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_returns_leaf_refusal_f64,
    Display::Grid,
    f64,
    assert_overflow_grid_refusal,
    true
);

deep_grid_observation_test!(
    deep_grid_lanes_overflow_observation_preserves_geometry_f32,
    Display::GridLanes,
    f32,
    assert_overflow_grid_geometry,
    false
);
deep_grid_observation_test!(
    deep_grid_lanes_overflow_observation_preserves_geometry_f64,
    Display::GridLanes,
    f64,
    assert_overflow_grid_geometry,
    false
);
deep_grid_observation_test!(
    deep_grid_lanes_overflow_observation_returns_leaf_refusal_f32,
    Display::GridLanes,
    f32,
    assert_overflow_grid_refusal,
    false
);
deep_grid_observation_test!(
    deep_grid_lanes_overflow_observation_returns_leaf_refusal_f64,
    Display::GridLanes,
    f64,
    assert_overflow_grid_refusal,
    false
);

deep_grid_observation_test!(
    deep_grid_lanes_overflow_hidden_subtree_preserves_geometry_f32,
    Display::GridLanes,
    f32,
    assert_overflow_grid_geometry,
    true
);
deep_grid_observation_test!(
    deep_grid_lanes_overflow_hidden_subtree_preserves_geometry_f64,
    Display::GridLanes,
    f64,
    assert_overflow_grid_geometry,
    true
);
deep_grid_observation_test!(
    deep_grid_lanes_overflow_hidden_subtree_returns_leaf_refusal_f32,
    Display::GridLanes,
    f32,
    assert_overflow_grid_refusal,
    true
);
deep_grid_observation_test!(
    deep_grid_lanes_overflow_hidden_subtree_returns_leaf_refusal_f64,
    Display::GridLanes,
    f64,
    assert_overflow_grid_refusal,
    true
);

fn assert_overflow_grid_distinct_request_work<S: LayoutScalar>(container: Display) {
    let mut traces = Vec::new();
    for hidden_subtree in [false, true] {
        for depth in [2, 3] {
            let tree =
                overflow_grid_chain::<S>(container, depth, hidden_subtree, valid_measurement::<S>);
            let (batch, admissions) =
                crate::engine::compute_ordinary_with_request_trace(&tree, 0, standalone())
                    .expect("ordinary container request trace completes");
            assert_overflow_grid_batch_geometry(&batch, depth, hidden_subtree);
            for node in 0..=depth {
                assert!(
                    admissions.iter().any(|row| row.node == node),
                    "every visible root, container and terminal has actual body work: depth={depth}, hidden_subtree={hidden_subtree}, node={node}"
                );
            }
            // Each nested container (not the root, which runs as the root
            // request) is size-computed with automatic scrollbars, which is
            // the size-only observation under test. Asserting its admission
            // shows the observation is reachable for the container display.
            for node in 1..depth {
                assert!(
                    admissions.iter().any(|row| row.node == node
                        && row.input.run_mode() == crate::output::RunMode::ComputeSize),
                    "every nested container has a size-only admission: container={container:?}, depth={depth}, hidden_subtree={hidden_subtree}, node={node}"
                );
            }
            assert!(
                admissions.iter().all(|row| row.node <= depth),
                "hidden traversal bypasses actual body admission"
            );
            traces.push((depth, hidden_subtree, admissions));
        }
    }
    for (depth, hidden_subtree, admissions) in traces {
        for (index, admission) in admissions.iter().enumerate() {
            let earlier = admissions[..index]
                .iter()
                .find(|row| row.same_request(admission));
            assert!(
                earlier.is_none(),
                "ordinary completed work must run once per full request identity: depth={depth}, hidden_subtree={hidden_subtree}, repeated={admission:?}"
            );
        }
    }
}

#[test]
fn grid_overflow_work_runs_once_per_distinct_request_f32() {
    assert_overflow_grid_distinct_request_work::<f32>(Display::Grid);
}

#[test]
fn grid_overflow_work_runs_once_per_distinct_request_f64() {
    assert_overflow_grid_distinct_request_work::<f64>(Display::Grid);
}

#[test]
fn grid_lanes_overflow_work_runs_once_per_distinct_request_f32() {
    assert_overflow_grid_distinct_request_work::<f32>(Display::GridLanes);
}

#[test]
fn grid_lanes_overflow_work_runs_once_per_distinct_request_f64() {
    assert_overflow_grid_distinct_request_work::<f64>(Display::GridLanes);
}

// Grid Lanes chain whose every container has horizontal overflow. Level `k`
// is `60 + 10k` px wide with one lane of that width and a max-content block
// size, and its only item (level `k + 1`) is 10px wider than the lane. The
// terminal is a visible 40px-high box of the deepest width.
fn scrollbar_lanes_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new();
    for node in 0..=depth {
        let terminal = node == depth;
        let width = S::from_f64(60.0 + 10.0 * f64::from(node));
        tree = tree.style(
            node,
            NodeInputOf {
                display: if terminal {
                    Display::Block
                } else {
                    Display::GridLanes
                },
                size: Size::new(
                    PreferredSizeOf::px(width),
                    if terminal {
                        PreferredSizeOf::px(S::from_f64(40.0))
                    } else {
                        PreferredSizeOf::MAX_CONTENT
                    },
                ),
                grid_template_columns: if terminal {
                    Vec::new()
                } else {
                    vec![crate::value::TrackComponentOf::from(
                        crate::value::LengthPercentageOf::px(width).unwrap(),
                    )]
                },
                overflow: if terminal {
                    ComputedOverflow::VISIBLE
                } else {
                    ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap()
                },
                scrollbar_width: crate::node_input::ScrollbarWidthOf::try_new(S::from_f64(1.0))
                    .unwrap(),
                ..NodeInputOf::default()
            },
        );
        if !terminal {
            tree = tree.children(node, [node + 1]);
        }
    }
    tree.measure_with(depth, valid_measurement::<S>)
}

fn assert_scrollbar_lanes_geometry<S: LayoutScalar>(depth: u32) {
    let tree = scrollbar_lanes_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("scrollbar Grid Lanes layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 1);
    for (node, layout) in batch.node_layouts() {
        let output = layout.single_box().expect("visible box").final_output();
        // Definite widths: 60px at the root, 10px more per level (CSS Box
        // Sizing 3, width). The single lane places its item at the content
        // box origin (CSS Grid 3, grid lanes).
        let width = 60.0 + 10.0 * f64::from(node);
        // The terminal is 40px high. Every other container sizes its block
        // axis to its content (max-content), which is the item's border-box
        // height plus the horizontal scrollbar that the item's 10px of
        // inline overflow creates, because overflow:auto shows a scrollbar
        // when content overflows the padding box (CSS Overflow 3, overflow).
        // The scrollbar is 1px thick and sits inside the border box, so each
        // level adds exactly 1px: 40 + (depth - node).
        let height = 40.0 + f64::from(depth - node);
        assert_eq!(
            output.size,
            Size::new(S::from_f64(width), S::from_f64(height)),
            "node {node}"
        );
        assert_eq!(output.location, Point::ZERO, "node {node}");
        // A horizontal scrollbar reserves block-axis space only; no
        // vertical scrollbar is needed because the block size fits its
        // content. The terminal has overflow:visible and no scrollbar.
        assert_eq!(
            output.scrollbar_size(),
            if node == depth {
                Size::ZERO
            } else {
                Size::new(S::ZERO, S::from_f64(1.0))
            },
            "node {node}"
        );
    }
}

#[test]
fn shallow_grid_lanes_scrollbar_overflow_preserves_geometry() {
    on_bounded_stack(
        "engine::depth_tests::grid_overflow_observation::shallow_grid_lanes_scrollbar_overflow_preserves_geometry",
        || {
            assert_scrollbar_lanes_geometry::<f32>(2);
            assert_scrollbar_lanes_geometry::<f64>(2);
        },
    );
}

#[test]
fn deep_grid_lanes_scrollbar_overflow_preserves_geometry_f32() {
    on_bounded_stack(
        "engine::depth_tests::grid_overflow_observation::deep_grid_lanes_scrollbar_overflow_preserves_geometry_f32",
        || assert_scrollbar_lanes_geometry::<f32>(24),
    );
}

#[test]
fn deep_grid_lanes_scrollbar_overflow_preserves_geometry_f64() {
    on_bounded_stack(
        "engine::depth_tests::grid_overflow_observation::deep_grid_lanes_scrollbar_overflow_preserves_geometry_f64",
        || assert_scrollbar_lanes_geometry::<f64>(24),
    );
}
