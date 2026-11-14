use super::*;

fn valid_measurement<S: LayoutScalar>(_: crate::LeafMeasureInputOf<S>) -> Size<S> {
    Size::new(S::from_f64(60.0), S::from_f64(40.0))
}

fn refuse_final_placement<S: LayoutScalar>(input: crate::LeafMeasureInputOf<S>) -> Size<S> {
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
    depth: u32,
    hidden_subtree: bool,
    measure: fn(crate::LeafMeasureInputOf<S>) -> Size<S>,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new();
    for node in 0..=depth {
        let terminal = node == depth;
        tree = tree.style(
            node,
            NodeInputOf {
                display: if terminal {
                    Display::Block
                } else {
                    Display::Grid
                },
                size: Size::new(
                    crate::PreferredSizeOf::px(S::from_f64(60.0)),
                    if terminal {
                        crate::PreferredSizeOf::px(S::from_f64(40.0))
                    } else {
                        crate::PreferredSizeOf::MAX_CONTENT
                    },
                ),
                overflow: if terminal {
                    crate::ComputedOverflow::VISIBLE
                } else {
                    crate::ComputedOverflow::try_new(crate::Overflow::Auto, crate::Overflow::Auto)
                        .unwrap()
                },
                scrollbar_width: crate::ScrollbarWidthOf::try_new(S::from_f64(1.0)).unwrap(),
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
        // One supported hidden subtree is reachable from every visible Grid.
        // Its nonleaf hidden parent excludes completed-request reuse without
        // adding any visible Grid levels or descendant body layout.
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

fn assert_overflow_grid_geometry<S: LayoutScalar>(depth: u32, hidden_subtree: bool) {
    let tree = overflow_grid_chain::<S>(depth, hidden_subtree, valid_measurement::<S>);
    let batch = compute_layout(&tree, 0, standalone()).expect("automatic-overflow Grid layout");
    assert_eq!(
        batch.node_layouts().len(),
        (2 * depth + 1 + u32::from(hidden_subtree)) as usize
    );
    // One visible 60x40 item fills the default automatic track on each axis.
    // Max-content block sizing encloses that item exactly. No edges, margins,
    // gaps or positioned content can require an actual automatic scrollbar.
    for (node, layout) in batch.node_layouts() {
        if node > depth {
            let hidden = layout.non_box().expect("hidden sibling");
            assert_eq!(hidden.kind(), crate::NonBoxKind::Hidden);
            assert_eq!(
                hidden.source_index(),
                crate::SourceIndex::new(usize::from(node <= 2 * depth))
            );
            assert_eq!(hidden.final_geometry().location, crate::Point::ZERO);
            assert_eq!(hidden.final_geometry().size, Size::ZERO);
        } else {
            let output = layout.single_box().expect("visible box").final_output();
            assert_eq!(
                output.size,
                Size::new(S::from_f64(60.0), S::from_f64(40.0)),
                "node {node}"
            );
            assert_eq!(output.location, crate::Point::ZERO, "node {node}");
            assert_eq!(output.source_index, crate::SourceIndex::ZERO, "node {node}");
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

fn assert_overflow_grid_refusal<S: LayoutScalar>(depth: u32, hidden_subtree: bool) {
    let tree = overflow_grid_chain::<S>(depth, hidden_subtree, refuse_final_placement::<S>);
    let error = compute_layout(&tree, 0, standalone()).expect_err("invalid final leaf output");
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(depth));
    assert_eq!(error.operation(), crate::LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::MeasurementOutput(output))
            if output.axis() == crate::PhysicalAxis::Vertical
                && matches!(output.error(), crate::NonNegativeFiniteScalarErrorOf::NonFinite { value } if value.to_f64().is_nan())
    ));
}

#[test]
fn shallow_grid_overflow_observation_preserves_geometry_and_refusal() {
    on_bounded_stack(
        "engine::depth_tests::grid_overflow_observation::shallow_grid_overflow_observation_preserves_geometry_and_refusal",
        || {
            for hidden_subtree in [false, true] {
                assert_overflow_grid_geometry::<f32>(2, hidden_subtree);
                assert_overflow_grid_refusal::<f32>(2, hidden_subtree);
                assert_overflow_grid_geometry::<f64>(2, hidden_subtree);
                assert_overflow_grid_refusal::<f64>(2, hidden_subtree);
            }
        },
    );
}

macro_rules! deep_grid_observation_test {
    ($name:ident, $scalar:ty, $check:ident, $hidden_subtree:expr) => {
        #[test]
        fn $name() {
            on_bounded_stack(
                concat!(
                    "engine::depth_tests::grid_overflow_observation::",
                    stringify!($name)
                ),
                || $check::<$scalar>(24, $hidden_subtree),
            );
        }
    };
}

deep_grid_observation_test!(
    deep_grid_overflow_observation_preserves_geometry_f32,
    f32,
    assert_overflow_grid_geometry,
    false
);
deep_grid_observation_test!(
    deep_grid_overflow_observation_preserves_geometry_f64,
    f64,
    assert_overflow_grid_geometry,
    false
);
deep_grid_observation_test!(
    deep_grid_overflow_observation_returns_leaf_refusal_f32,
    f32,
    assert_overflow_grid_refusal,
    false
);
deep_grid_observation_test!(
    deep_grid_overflow_observation_returns_leaf_refusal_f64,
    f64,
    assert_overflow_grid_refusal,
    false
);

deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_preserves_geometry_f32,
    f32,
    assert_overflow_grid_geometry,
    true
);
deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_preserves_geometry_f64,
    f64,
    assert_overflow_grid_geometry,
    true
);
deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_returns_leaf_refusal_f32,
    f32,
    assert_overflow_grid_refusal,
    true
);
deep_grid_observation_test!(
    deep_grid_overflow_hidden_subtree_returns_leaf_refusal_f64,
    f64,
    assert_overflow_grid_refusal,
    true
);
