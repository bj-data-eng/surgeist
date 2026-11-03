//! Supplemental functional controls; child of native_maximum_bound.rs.
use super::*;
use crate::{ScrollbarGutter, ScrollbarWidthOf};

fn maximum_source<S: LayoutScalar>(maximum: f64, cap: GrowthCap) -> DistinctLeaf<S> {
    let mut tree = source::<S>(20.0, cap, Overflow::Hidden);
    let mut root = tree.node_input(0).clone();
    root.display = Display::InlineGrid;
    root.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
    root.max_size.width = MaxSizeOf::px(S::from_f64(maximum));
    root.grid_template_rows = vec![TrackComponentOf::AUTO];
    tree.source.insert_input(0, LayoutInputOf::box_input(root));
    let mut child = tree.node_input(1).clone();
    child.size.height = PreferredSizeOf::AUTO;
    tree.source.insert_input(1, LayoutInputOf::box_input(child));
    qualify_actual_native_queries(&tree);
    tree
}

fn formal_input<S: LayoutScalar>(known_width: Option<S>) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(known_width, None),
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}

fn actual_formal<S: LayoutScalar>(
    tree: &DistinctLeaf<S>,
    known_width: Option<S>,
) -> ComputeOutputOf<S> {
    tree.observed.borrow_mut().clear();
    let mut session = crate::engine::ComputeSession::new(tree, Vec::new());
    session
        .compute_child(0, formal_input(known_width))
        .expect("the genuine native Grid query completes")
}

fn inactive_and_active_finishing_budget<S: LayoutScalar>() {
    let n = S::from_f64;
    // A larger maximum does not allocate100 or replace the MinContent floor40.
    let inactive = maximum_source::<S>(100.0, GrowthCap::MinContent);
    let output = actual_formal(&inactive, None);
    assert_eq!(output.size, Size::new(n(40.0), n(10.0)));
    assert_height_queries_with_opposite_width(
        &inactive.observed.borrow(),
        n(40.0),
        "inactive maximum100",
    );

    // This real MaxContent cap distinguishes the intrinsic floor40 from
    // growth80: maximum60 is an active finishing budget, not a phase reset.
    let active = maximum_source::<S>(60.0, GrowthCap::MaxContent);
    let output = actual_formal(&active, None);
    assert_eq!(output.size, Size::new(n(60.0), n(10.0)));
    assert_height_queries_with_opposite_width(
        &active.observed.borrow(),
        n(60.0),
        "active maximum60 finishing budget",
    );
}

fn exhausted_scrollbar_budget<S: LayoutScalar>() {
    let n = S::from_f64;
    let mut tree = maximum_source::<S>(20.0, GrowthCap::MinContent);
    let mut root = tree.node_input(0).clone();
    root.overflow = computed_overflow(Overflow::Hidden, Overflow::Hidden);
    root.scrollbar_gutter = ScrollbarGutter::StableBothEdges;
    root.scrollbar_width = ScrollbarWidthOf::try_new(n(15.0)).unwrap();
    tree.source.insert_input(0, LayoutInputOf::box_input(root));

    // Two requested15px inline gutters exceed max20. The finishing budget
    // saturates to0; it cannot erase the intrinsic40px track floor.
    let output = actual_formal(&tree, None);
    assert_eq!(output.size, Size::new(n(20.0), n(10.0)));
    assert_height_queries_with_opposite_width(
        &tree.observed.borrow(),
        n(40.0),
        "exhausted intrinsic finishing budget",
    );

    // A separate real normal allocation20 has effective20px total gutters
    // and zero content width. These are the actual canonical scroll views.
    let normal = compute_layout(&tree, 0, {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(n(20.0)).unwrap()),
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
    })
    .expect("the genuine saturated normal Grid completes");
    for entries in [normal.unrounded_entries(), normal.final_entries()] {
        let output = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(output.size, Size::new(n(20.0), n(10.0)));
        let scroll = output
            .scroll_geometry
            .expect("normal root has actual scroll geometry");
        assert_eq!(scroll.scrollbar_size(), Size::new(n(20.0), S::ZERO));
        assert_eq!(scroll.content_box().size(), Size::new(S::ZERO, n(10.0)));
    }
}

#[test]
fn active_and_inactive_maximum_finishing_controls_both_scalars() {
    inactive_and_active_finishing_budget::<f32>();
    inactive_and_active_finishing_budget::<f64>();
}
#[test]
fn saturated_maximum_scrollbar_budget_keeps_intrinsic_phases_both_scalars() {
    exhausted_scrollbar_budget::<f32>();
    exhausted_scrollbar_budget::<f64>();
}
