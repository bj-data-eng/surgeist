//! A Grid own maximum does not replace its intrinsic opposite-axis content basis.
use super::*;
use crate::engine::contracts::Compute;
use crate::test_support::readback::{self, Phase};

fn assert_height_queries_with_opposite_width<S: LayoutScalar>(
    inputs: &[LeafMeasureInputOf<S>],
    width: S,
    operation: &str,
) {
    for height in [SizingOperation::MinContent, SizingOperation::MaxContent] {
        assert!(
            inputs.iter().any(|input| {
                let available = input.content_sizing_request().axes();
                input.known_content_size() == Size::new(Some(width), None)
                    && available
                        .width
                        .space()
                        .definite_size()
                        .is_some_and(|value| value.get() == width)
                    && available.height.operation() == height
            }),
            "{operation}: actual {height:?} height query keeps opposite width{width:?}; receipts={inputs:?}"
        );
    }
}

fn intrinsic_maximum_preserves_opposite_width<S: LayoutScalar>(
    width_operation: AxisSizingRequestOf<S>,
) {
    let operation = match width_operation.operation() {
        SizingOperation::MinContent => "formal min-content operation",
        SizingOperation::MaxContent => "formal max-content operation",
        SizingOperation::Normal => panic!("this fixture qualifies intrinsic width operations"),
    };
    // Reuse the maintained native40/80 host; only its authored box inputs change.
    let mut tree = source::<S>(20.0, GrowthCap::MinContent, Overflow::Hidden);
    let mut root = tree.node_input(0).clone();
    root.display = Display::InlineGrid;
    root.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
    root.max_size.width = MaxSizeOf::px(S::from_f64(20.0));
    root.grid_template_rows = vec![TrackComponentOf::AUTO];
    tree.source.insert_input(0, LayoutInputOf::box_input(root));
    let mut child = tree.node_input(1).clone();
    child.size.height = PreferredSizeOf::AUTO;
    tree.source.insert_input(1, LayoutInputOf::box_input(child));
    assert!(!tree.has_leaf_measurement(0));
    assert!(tree.has_leaf_measurement(1));
    assert_eq!(
        tree.node_input(0).size,
        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
    );
    assert_eq!(
        tree.node_input(0).max_size.width,
        MaxSizeOf::px(S::from_f64(20.0))
    );
    assert_eq!(
        tree.node_input(0).grid_template_columns,
        vec![GrowthCap::MinContent.track()]
    );
    assert_eq!(
        tree.node_input(0).grid_template_rows,
        vec![TrackComponentOf::AUTO]
    );
    assert_eq!(
        tree.node_input(1).size,
        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
    );
    qualify_actual_native_queries(&tree);

    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let minimum = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::ZERO), None),
        Size::NONE,
        ContainingLayoutContext::new(flow, ParentFormattingContext::Grid),
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    let minimum_output =
        crate::measurement::compute_tree_leaf(&tree, 1, minimum, tree.node_input(1))
            .expect("the real hidden native leaf accepts its zero-minimum query");
    assert_eq!(minimum_output.size, Size::new(S::ZERO, S::from_f64(10.0)));
    assert!(
        tree.observed
            .borrow()
            .iter()
            .any(|input| input.known_content_size() == Size::new(Some(S::ZERO), None))
    );

    // This is a root/host intrinsic query. Its measured node being a Grid does
    // not make its incoming parent a Grid; only native child traversal does so.
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
        SizingRequestOf::new(
            Size::new(
                width_operation,
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    assert_eq!(input.known(), Size::NONE);
    assert_eq!(input.parent(), Size::NONE);
    assert_eq!(
        input.sizing_request().axes(),
        Size::new(
            width_operation,
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent)
        )
    );
    assert_eq!(
        input.containing_layout_context(),
        ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent)
    );
    assert_eq!(input.run_mode(), RunMode::ComputeSize);
    assert_eq!(input.sizing_mode(), SizingMode::InherentSize);
    tree.observed.borrow_mut().clear();
    let intrinsic_output = {
        let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
        session
            .compute_child(0, input)
            .expect("the Grid intrinsic operation completes")
    };
    let intrinsic_receipts = tree.observed.borrow().clone();
    assert!(
        intrinsic_receipts.iter().any(|input| {
            input.known_content_size() == Size::NONE
                && input.content_sizing_request().axes().width.operation()
                    == width_operation.operation()
        }),
        "{operation}: the actual responsive leaf receives this intrinsic width with unknown dimensions"
    );
    tree.observed.borrow_mut().clear();

    // A supplied own width20 remains authoritative even when the independent
    // available-space operation is intrinsic. This is a separate execution.
    let known_input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(20.0)), None),
        Size::NONE,
        ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
        SizingRequestOf::new(
            Size::new(
                width_operation,
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    assert_eq!(
        known_input.known(),
        Size::new(Some(S::from_f64(20.0)), None)
    );
    assert_eq!(
        known_input.sizing_request().axes(),
        Size::new(
            width_operation,
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent)
        )
    );
    let known_output = {
        let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
        session
            .compute_child(0, known_input)
            .expect("the Grid own-known-width operation completes")
    };
    let known_receipts = tree.observed.borrow().clone();
    assert_eq!(
        known_output.size,
        Size::new(S::from_f64(20.0), S::from_f64(10.0)),
        "{operation}: own-known root node0 border size"
    );
    assert_height_queries_with_opposite_width(
        &known_receipts,
        S::from_f64(20.0),
        "supplied own-known allocation",
    );
    tree.observed.borrow_mut().clear();

    // A separate accepted normal width20 remains a real full-layout control.
    let normal_request = {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(S::from_f64(20.0)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    };
    let normal =
        compute_layout(&tree, 0, normal_request).expect("the accepted-width normal Grid completes");
    let border_size = Size::new(S::from_f64(20.0), S::from_f64(10.0));
    let native_content_size = Size::new(S::from_f64(40.0), S::from_f64(10.0));
    for (phase, entries) in [
        (
            "unrounded",
            readback::continuous_custody(&normal, Phase::Unrounded).as_slice(),
        ),
        (
            "final",
            readback::continuous_custody(&normal, Phase::Final).as_slice(),
        ),
    ] {
        for node in [0, 1] {
            let actual = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap_or_else(|| panic!("{phase} node {node}: missing native box"))
                .output();
            assert_eq!(actual.size, border_size, "{phase} node {node}: border size");
            assert_eq!(
                actual.location,
                Point::ZERO,
                "{phase} node {node}: location"
            );
            if node == 1 {
                // Normal leaf layout passes unknown content dimensions with
                // definite20 space. The native provider still measures its
                // minimum40 content inside the accepted20 border.
                assert_eq!(
                    actual.content_size, native_content_size,
                    "{phase} node {node}: native measured content size"
                );
            }
            assert_eq!(
                actual.content_box_size(),
                border_size,
                "{phase} node {node}: canonical content-box size"
            );
            assert_eq!(
                actual.source_index,
                SourceIndex::new(0),
                "{phase} node {node}: source index"
            );
            assert_eq!(
                (actual.margin, actual.border, actual.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO),
                "{phase} node {node}: box edges"
            );
            let scroll = actual
                .scroll_geometry
                .unwrap_or_else(|| panic!("{phase} node {node}: missing scroll geometry"));
            assert_eq!(scroll.flow_axes(), flow, "{phase} node {node}: scroll axes");
            for (field, rect) in [
                ("border box", scroll.border_box()),
                ("padding box", scroll.padding_box()),
                ("content box", scroll.content_box()),
                ("scrollport", scroll.scrollport()),
            ] {
                assert_eq!(
                    rect.origin(),
                    Point::ZERO,
                    "{phase} node {node}: {field} origin"
                );
                assert_eq!(
                    rect.size(),
                    border_size,
                    "{phase} node {node}: {field} size"
                );
            }
            // Hidden retains the leaf's own measured overflow and programmatic
            // range, while preventing that descendant overflow from reaching
            // the Grid. Its accepted child border remains20 in both owners.
            let overflow_size = if node == 1 {
                native_content_size
            } else {
                border_size
            };
            assert_eq!(
                scroll.scrollable_overflow().origin(),
                Point::ZERO,
                "{phase} node {node}: scrollable-overflow origin"
            );
            assert_eq!(
                scroll.scrollable_overflow().size(),
                overflow_size,
                "{phase} node {node}: scrollable-overflow size"
            );
            assert_eq!(
                scroll.scrollbar_size(),
                Size::ZERO,
                "{phase} node {node}: scrollbars"
            );
            let range = scroll.physical_range();
            let horizontal_end = if node == 1 {
                S::from_f64(20.0)
            } else {
                S::ZERO
            };
            assert_eq!(
                (
                    range.x().minimum(),
                    range.x().maximum(),
                    range.y().minimum(),
                    range.y().maximum()
                ),
                (S::ZERO, horizontal_end, S::ZERO, S::ZERO),
                "{phase} node {node}: physical scroll range"
            );
        }
    }
    let normal_receipts = tree.observed.borrow().clone();
    assert_height_queries_with_opposite_width(
        &normal_receipts,
        S::from_f64(20.0),
        "accepted normal allocation",
    );
    assert_eq!(
        intrinsic_output.size,
        Size::new(S::from_f64(20.0), S::from_f64(10.0)),
        "{operation}: intrinsic root node0 maximum-clamped border size"
    );

    // Presence of these interactions observes the content-derived
    // opposite width without identifying every provisional query as a stage.
    assert_height_queries_with_opposite_width(&intrinsic_receipts, S::from_f64(40.0), operation);
}

#[test]
fn native_grid_max_content_keeps_content_opposite_width_under_maximum_f32() {
    intrinsic_maximum_preserves_opposite_width::<f32>(AxisSizingRequestOf::new(
        AvailableSpaceOf::Indefinite,
        SizingOperation::MaxContent,
    ));
}
#[test]
fn native_grid_max_content_keeps_content_opposite_width_under_maximum_f64() {
    intrinsic_maximum_preserves_opposite_width::<f64>(AxisSizingRequestOf::new(
        AvailableSpaceOf::Indefinite,
        SizingOperation::MaxContent,
    ));
}

#[test]
fn native_grid_min_content_keeps_content_opposite_width_under_maximum_f32() {
    intrinsic_maximum_preserves_opposite_width::<f32>(AxisSizingRequestOf::new(
        AvailableSpaceOf::Indefinite,
        SizingOperation::MinContent,
    ));
}
#[test]
fn native_grid_min_content_keeps_content_opposite_width_under_maximum_f64() {
    intrinsic_maximum_preserves_opposite_width::<f64>(AxisSizingRequestOf::new(
        AvailableSpaceOf::Indefinite,
        SizingOperation::MinContent,
    ));
}

#[path = "native_maximum_bound_controls.rs"]
mod controls;

// A finite physical containing space does not replace a formal operation.
fn finite_space_intrinsic_maximum<S: LayoutScalar>(operation: SizingOperation) {
    intrinsic_maximum_preserves_opposite_width::<S>(AxisSizingRequestOf::new(
        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(S::from_f64(20.0)).unwrap()),
        operation,
    ));
}
#[test]
fn finite_space_formal_minimum_f32() {
    finite_space_intrinsic_maximum::<f32>(SizingOperation::MinContent);
}
#[test]
fn finite_space_formal_minimum_f64() {
    finite_space_intrinsic_maximum::<f64>(SizingOperation::MinContent);
}
#[test]
fn finite_space_formal_maximum_f32() {
    finite_space_intrinsic_maximum::<f32>(SizingOperation::MaxContent);
}
#[test]
fn finite_space_formal_maximum_f64() {
    finite_space_intrinsic_maximum::<f64>(SizingOperation::MaxContent);
}
