//! A cyclic subgrid gap is zero for intrinsic sizing, then uses its own content box.
use super::*;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    let mut entries = entries.iter().filter(|entry| entry.node() == node);
    let value = entries
        .next()
        .expect("the real native box is published")
        .output();
    assert!(entries.next().is_none(), "one native box for node{node}");
    value
}

fn assert_native_cyclic_subgrid_gap<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(PreferredSizeOf::px(n(100.0)), PreferredSizeOf::px(n(20.0))),
                grid_template_columns: vec![TrackComponentOf::AUTO; 3],
                grid_template_rows: vec![TrackComponentOf::px(n(20.0))],
                gap: Size::new(LengthOf::px(n(10.0)), LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                grid_column: GridPlacement::try_lines(1, 4).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: vec![TrackComponentOf::px(n(20.0))],
                gap: Size::new(LengthOf::percent(n(0.2)), LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_self: Some(AlignItems::Stretch),
                align_self: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(PreferredSizeOf::px(n(40.0)), PreferredSizeOf::px(n(10.0))),
                grid_column: GridPlacement::try_lines(2, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        );
    assert_eq!(Traverse::children(&tree, 0).collect::<Vec<_>>(), [1]);
    assert_eq!(Traverse::children(&tree, 1).collect::<Vec<_>>(), [2]);
    assert_eq!(Traverse::child_count(&tree, 2), 0);
    for node in [0, 1, 2] {
        assert!(
            !tree.has_leaf_measurement(node),
            "the fixture uses only native layout"
        );
    }
    assert_eq!(
        tree.node_input(1).grid_column,
        GridPlacement::try_lines(1, 4).unwrap()
    );
    assert_eq!(tree.node_input(1).gap.width, LengthOf::percent(n(0.2)));
    assert_eq!(
        tree.node_input(2).size,
        Size::new(PreferredSizeOf::px(n(40.0)), PreferredSizeOf::px(n(10.0)))
    );

    let leaf_control = PublicLayoutTreeOf::<S>::new()
        .children(2, [])
        .style(2, tree.node_input(2).clone());
    for available in [
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
    ] {
        let control = compute_layout(&leaf_control, 2, {
            let axes = Size::splat(available);
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
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                definite,
            )
        })
        .expect("the fixed native leaf has intrinsic geometry");
        for entries in [control.unrounded_entries(), control.final_entries()] {
            assert_eq!(output(entries, 2).size, Size::new(n(40.0), n(10.0)));
        }
    }

    let batch = compute_layout(&tree, 0, {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(n(100.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(n(20.0)).unwrap()),
                SizingOperation::Normal,
            ),
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
    .expect("the real native cyclic-gap subgrid completes");

    // Align3 §8.3: intrinsic gap0 makes each interior half-gap (0-10)/2=-5.
    // Grid2 subgrid-gaps: the middle contribution is40-5-5=30, giving
    // parent tracks0/30/0 plus two10px gutters, a50px borrowed span.
    // Normal layout resolves20% against that50: gap10, no gap difference
    // and no backward resize. The fixed40px child starts at10 and ends at50.
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        for (node, x, width, height) in [
            (0, 0.0, 100.0, 20.0),
            (1, 0.0, 50.0, 20.0),
            (2, 10.0, 40.0, 10.0),
        ] {
            let actual = output(entries, node);
            assert_eq!(
                actual.location,
                Point::new(n(x), S::ZERO),
                "node{node} normal position"
            );
            assert_eq!(
                actual.size,
                Size::new(n(width), n(height)),
                "node{node} normal size"
            );
            assert_eq!(actual.content_box_size(), actual.size);
            assert_eq!(actual.source_index, SourceIndex::new(0));
            assert_eq!(
                (actual.margin, actual.border, actual.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO)
            );
            let scroll = actual
                .scroll_geometry
                .expect("performed native box retains scroll geometry");
            assert_eq!(
                scroll.flow_axes(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
            );
            assert_eq!(scroll.used_overflow_x(), Overflow::Visible);
            assert_eq!(scroll.used_overflow_y(), Overflow::Visible);
            for rect in [
                scroll.border_box(),
                scroll.padding_box(),
                scroll.content_box(),
                scroll.scrollport(),
                scroll.scrollable_overflow(),
            ] {
                assert_eq!(rect.origin(), Point::ZERO, "node{node} local scroll origin");
                assert_eq!(
                    rect.size(),
                    Size::new(n(width), n(height)),
                    "node{node} own-border scroll extent"
                );
            }
            assert_eq!(scroll.scrollbar_size(), Size::ZERO);
            let range = scroll.physical_range();
            assert_eq!(
                (
                    range.x().minimum(),
                    range.x().maximum(),
                    range.y().minimum(),
                    range.y().maximum()
                ),
                (S::ZERO, S::ZERO, S::ZERO, S::ZERO)
            );
        }
        let root = output(entries, 0);
        let subgrid = output(entries, 1);
        let leaf = output(entries, 2);
        assert_eq!(root.size.width - subgrid.size.width, n(50.0));
        assert_eq!(leaf.location.x + leaf.size.width, subgrid.size.width);
    }
}

#[test]
fn native_cyclic_subgrid_gap_keeps_intrinsic_zero_and_final_own_basis_f32() {
    assert_native_cyclic_subgrid_gap::<f32>();
}

#[test]
fn native_cyclic_subgrid_gap_keeps_intrinsic_zero_and_final_own_basis_f64() {
    assert_native_cyclic_subgrid_gap::<f64>();
}
