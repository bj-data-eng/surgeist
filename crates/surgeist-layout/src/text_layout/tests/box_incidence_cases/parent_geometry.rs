//! Private child of text_layout::tests::box_incidence_cases.
//! Genuine captured empty Inline between two original Ahem10 occurrences.
use super::*;

fn lane<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let case = Case::new(Shape::Paired, direction);
        let scope = &case.scopes[0];
        let start = scope.entry_control().unwrap();
        let end = scope.exit_control().unwrap();
        assert!(start.event_position() < end.event_position());
        assert_eq!(
            (start.source_position().get(), end.source_position().get()),
            (1, 1)
        );
        assert_eq!(
            case.binding
                .prepared
                .analyzed_source()
                .source()
                .text()
                .code_units(),
            [65, 66]
        );
        let original_levels: Vec<_> = case
            .binding
            .prepared
            .control_views()
            .map(|row| (row.handle(), row.level().get()))
            .collect();
        let mut tree = case.tree::<S>(25.0, false);
        for (node, kind, amount) in [
            (20, InlineBoundaryKind::Start, 2.0),
            (21, InlineBoundaryKind::End, 3.0),
        ] {
            let boundary = InlineBoundaryInputOf::box_edge(
                kind,
                InlineMetricsOf::try_new(InlineBaselineContext::Physical, S::ZERO, S::ZERO)
                    .unwrap(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(amount), S::ZERO).unwrap(),
                FlowAxes::new(WritingMode::HorizontalTb, direction),
            )
            .with_direction(direction);
            tree.insert_input(node, LayoutInputOf::inline_boundary(boundary));
        }
        let inventory = case.inventory(&[(WrapEdges::AUTO, None)], false, false);
        let flow =
            LayoutFlowInput::try_new(&tree, 0, vec![case.binding.clone()], inventory, Vec::new())
                .unwrap();
        let batch = compute_layout_with_flow(
            &flow,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(25.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap();
        let [placed] = batch.placed_text() else {
            panic!("one actual AB paragraph")
        };
        assert_eq!(placed.layout().lines().len(), 1);
        assert_eq!(placed.layout().source().text().code_units(), [65, 66]);
        let glyphs: Vec<_> = placed
            .layout()
            .run_views()
            .flat_map(|run| {
                let x = run.origin().x;
                run.glyphs()
                    .map(move |glyph| (glyph.id(), x + glyph.pen(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect();
        // Both Latin packets keep their actual strong-L source order. Only the
        // two sides of the authentic empty gap swap with the parent direction.
        assert_eq!(glyphs, [(35, 0.0, 10.0), (36, 15.0, 10.0)]);
        let line = batch
            .final_inline_fragments()
            .first()
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(line.feedback().advance, 25.0);
        assert_eq!(line.control_slots().count(), 2);
        let expected = if direction == Direction::Ltr {
            [(20, 10.0, 12.0), (21, 12.0, 15.0)]
        } else {
            [(20, 13.0, 15.0), (21, 10.0, 13.0)]
        };
        for (node, left, right) in expected {
            let control = case
                .binding
                .controls
                .iter()
                .find(|row| row.node == node)
                .unwrap();
            let slot = line
                .control_slots()
                .find(|slot| slot.control() == control.occurrence)
                .unwrap();
            assert_eq!(slot.strip_interval(), (left, right));
            let output = batch
                .unrounded_entries()
                .iter()
                .find(|row| row.node() == node)
                .unwrap()
                .output();
            assert_eq!(
                (output.location.x, output.size.width),
                (S::from_f64(left), S::from_f64(right - left))
            );
        }
        assert_eq!(
            case.binding
                .prepared
                .control_views()
                .map(|row| (row.handle(), row.level().get()))
                .collect::<Vec<_>>(),
            original_levels
        );
    }
}

#[test]
fn captured_empty_box_edges_use_the_authentic_original_gap_f32() {
    lane::<f32>();
}
#[test]
fn captured_empty_box_edges_use_the_authentic_original_gap_f64() {
    lane::<f64>();
}

fn native_gap_on_prepared_formatter<S: LayoutScalar>() {
    // Execute the maintained actual Ahem/captured-gap control before requesting
    // authority owned only by the font-free numeric adapter.
    lane::<S>();
    let case = Case::new(Shape::Paired, Direction::Ltr);
    let mut tree = case.tree::<S>(25.0, false);
    for (node, kind, amount) in [
        (20, InlineBoundaryKind::Start, 2.0),
        (21, InlineBoundaryKind::End, 3.0),
    ] {
        let boundary = InlineBoundaryInputOf::box_edge(
            kind,
            InlineMetricsOf::try_new(InlineBaselineContext::Physical, S::ZERO, S::ZERO).unwrap(),
            InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(amount), S::ZERO).unwrap(),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        );
        tree.insert_input(node, LayoutInputOf::inline_boundary(boundary));
    }
    let mut inventory = case.inventory(&[(WrapEdges::AUTO, None)], false, false);
    inventory.gaps.push(crate::InlineGapGroupInput {
        anchor: crate::InlineGapAnchor::Formatter {
            node: 0,
            edge: crate::InlineFormatterEdge::Start,
        },
        boxes: vec![InlineBoxRef::Scope(case.scopes[0].clone())],
    });
    let error = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![case.binding.clone()],
        inventory,
        Vec::new(),
    )
    .err()
    .expect("native gap authority cannot be admitted and ignored on a Prepared-backed formatter");
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::UnsupportedIncidence
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn native_gap_group_on_genuine_prepared_empty_box_refuses_f32() {
    native_gap_on_prepared_formatter::<f32>();
}
#[test]
fn native_gap_group_on_genuine_prepared_empty_box_refuses_f64() {
    native_gap_on_prepared_formatter::<f64>();
}
