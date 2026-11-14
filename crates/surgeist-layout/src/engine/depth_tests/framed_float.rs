use super::*;

fn framed_float_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let terminal = 2 * depth + 1;
    let mut tree = chain::<S>(Display::Block, terminal + 1);
    for node in 1..terminal {
        if node.is_multiple_of(2) {
            tree = tree.style(
                node,
                NodeInputOf {
                    display: Display::Block,
                    float: crate::Float::Left,
                    size: Size::new(
                        crate::PreferredSizeOf::px(S::from_f64(60.0)),
                        crate::PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            );
        }
    }
    tree.style(
        terminal,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                crate::PreferredSizeOf::AUTO,
                crate::PreferredSizeOf::px(S::from_f64(7.0)),
            ),
            ..NodeInputOf::default()
        },
    )
}

fn assert_framed_float_geometry<S: LayoutScalar>(depth: u32) {
    let tree = framed_float_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("framed floats must lay out");
    let terminal = 2 * depth + 1;
    assert_eq!(batch.node_layouts().len(), (terminal + 1) as usize);
    // Each float establishes its own formatting context and contains the
    // seven-unit descendant float extent. Shared ordinary wrappers exclude
    // that extent from their own height; the root contains the outer float.
    for (node, view) in batch.node_layouts() {
        let output = view.single_box().expect("Block box").final_output();
        let width = if node <= 1 { 100.0 } else { 60.0 };
        let height = if node == 0 || node == terminal || node.is_multiple_of(2) {
            7.0
        } else {
            0.0
        };
        assert_eq!(
            output.size,
            Size::new(S::from_f64(width), S::from_f64(height)),
            "node {node}"
        );
        assert_eq!(output.location, crate::Point::ZERO, "node {node}");
        assert_eq!(
            output.source_index,
            crate::SourceIndex::new(0),
            "node {node}"
        );
    }
}

fn assert_framed_float_refusal<S: LayoutScalar>(depth: u32) {
    let terminal = 2 * depth + 1;
    let tree = framed_float_chain::<S>(depth)
        .style(terminal, NodeInputOf::default())
        .measure(terminal, Size::new(S::from_f64(60.0), S::NAN));
    let error = compute_layout(&tree, 0, standalone()).unwrap_err();
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(terminal));
    assert_eq!(error.operation(), crate::LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::MeasurementOutput(output))
            if output.axis() == crate::PhysicalAxis::Vertical
                && matches!(output.error(), crate::NonNegativeFiniteScalarErrorOf::NonFinite { value } if value.to_f64().is_nan())
    ));
}

#[test]
fn shallow_framed_float_layout_preserves_geometry_and_refusal() {
    on_bounded_stack(
        "engine::depth_tests::framed_float::shallow_framed_float_layout_preserves_geometry_and_refusal",
        || {
            assert_framed_float_geometry::<f32>(2);
            assert_framed_float_refusal::<f32>(2);
            assert_framed_float_geometry::<f64>(2);
            assert_framed_float_refusal::<f64>(2);
        },
    );
}

macro_rules! deep_framed_float_test {
    ($name:ident, $scalar:ty, $check:ident) => {
        #[test]
        fn $name() {
            on_bounded_stack(
                concat!("engine::depth_tests::framed_float::", stringify!($name)),
                || $check::<$scalar>(24),
            );
        }
    };
}

deep_framed_float_test!(
    deep_framed_float_layout_preserves_geometry_f32,
    f32,
    assert_framed_float_geometry
);
deep_framed_float_test!(
    deep_framed_float_layout_preserves_geometry_f64,
    f64,
    assert_framed_float_geometry
);
deep_framed_float_test!(
    deep_framed_float_layout_returns_original_refusal_f32,
    f32,
    assert_framed_float_refusal
);
deep_framed_float_test!(
    deep_framed_float_layout_returns_original_refusal_f64,
    f64,
    assert_framed_float_refusal
);
