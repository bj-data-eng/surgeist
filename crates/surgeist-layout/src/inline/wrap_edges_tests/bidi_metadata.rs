//! Physical box edges preserve the real supplied atomic content order.
use super::*;

#[path = "bidi_metadata/closing_cut.rs"]
mod closing_cut;

pub(super) fn split_box_fixture<S: LayoutScalar>() -> Fixture<S> {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut f = Fixture::new(Direction::Ltr, 100.0);
    for index in 0..6u32 {
        if index == 2 {
            f.boundary(20, InlineBoundaryKind::Start, 3.0);
        }
        let node = 10 + index;
        f.atomic(
            node,
            WrapEdges::AUTO,
            InlineBreakOpportunity::Prohibited,
            false,
        );
        let participation = AtomicInlineParticipation::new(
            BidiLevel::try_new(u8::from(index >= 3)).unwrap(),
            InlineBreakOpportunity::Prohibited,
        );
        // Resolved atomic participation is an actual normalized Native input,
        // independent of the formatter's LTR direction (bidi_order_tests).
        let mut actual = crate::LayoutTree::node_input(&f.tree, node).clone();
        actual.atomic_inline_participation = Some(participation);
        assert_eq!(actual.direction, Direction::Ltr);
        assert_eq!(actual.writing_mode, WritingMode::HorizontalTb);
        assert_eq!(
            actual.size,
            Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0)))
        );
        f.tree.insert_input(node, LayoutInputOf::box_input(actual));
        let MixedInlineParticipantOf::Atomic {
            participation: stored,
            ..
        } = f.participants.last_mut().unwrap()
        else {
            unreachable!("the maintained fixture just appended this real atomic")
        };
        *stored = participation;
        if index == 3 {
            f.boundary(21, InlineBoundaryKind::End, 5.0);
        }
    }
    for (node, kind, width) in [
        (20, InlineBoundaryKind::Start, 3.0),
        (21, InlineBoundaryKind::End, 5.0),
    ] {
        let boundary = InlineBoundaryInputOf::box_edge(
            kind,
            metrics(),
            InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
            axes,
        )
        .with_writing_mode(WritingMode::HorizontalTb)
        .with_direction(Direction::Ltr);
        assert_eq!(boundary.parent_edge_axes(), Some(axes));
        assert_eq!(
            (boundary.writing_mode(), boundary.direction()),
            (WritingMode::HorizontalTb, Direction::Ltr)
        );
        f.tree
            .insert_input(node, LayoutInputOf::inline_boundary(boundary));
        // Existing private diagnostic receipt is coupled to the very same
        // public input; it receives the actual checked projection from f.numeric.
        let stored = f
            .participants
            .iter_mut()
            .find_map(|participant| match participant {
                MixedInlineParticipantOf::Boundary(control)
                    if control.source_index() == node as usize =>
                {
                    Some(control)
                }
                _ => None,
            })
            .unwrap();
        *stored = InlineBoundaryControlOf::new(
            node as usize,
            boundary.kind(),
            InlineFlowOf::new(
                boundary.writing_mode(),
                boundary.direction(),
                AvailableOf::definite(f.width),
            ),
            boundary.metrics(),
            InlineControlAlignment::Baseline,
            boundary.edge(),
            boundary.parent_edge_axes(),
        );
    }
    f.wrapping = WrappingInput {
        native_inline_boxes: Vec::new(),
        gaps: Vec::new(),
        scopes: vec![WrappingScopeInput::NativePair {
            start: 20,
            end: 21,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![
            order(
                GeneratedBoxRef::Node(0),
                vec![
                    child(GeneratedBoxRef::Node(10)),
                    child(GeneratedBoxRef::Node(11)),
                    child(pair(20, 21)),
                    child(GeneratedBoxRef::Node(14)),
                    child(GeneratedBoxRef::Node(15)),
                ],
            ),
            order(
                pair(20, 21),
                vec![
                    child(GeneratedBoxRef::Node(12)),
                    child(GeneratedBoxRef::Node(13)),
                ],
            ),
        ],
    };
    f.finish();
    assert_eq!(f.nodes, [10, 11, 20, 12, 13, 21, 14, 15]);
    for (index, node) in (10..16u32).enumerate() {
        let actual = crate::LayoutTree::node_input(&f.tree, node);
        assert_eq!(
            actual
                .atomic_inline_participation
                .unwrap()
                .bidi_level()
                .get(),
            u8::from(index >= 3)
        );
        assert_eq!(
            actual.margin,
            Edges::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO).map(LengthAutoOf::px)
        );
    }
    let identities = f
        .participants
        .iter()
        .map(|participant| match participant {
            MixedInlineParticipantOf::Atomic { item, .. } => item.source_index,
            MixedInlineParticipantOf::Boundary(control) => control.source_index(),
            MixedInlineParticipantOf::ForcedLineBreak(_) => {
                panic!("there is no forced line boundary")
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(identities, [10, 11, 20, 12, 13, 21, 14, 15]);
    let flow = f.flow();
    assert!(
        flow.paragraphs().is_empty(),
        "no fabricated captured/PreparedText/font authority"
    );
    drop(flow);
    f
}

fn box_edges_preserve_atomic_order<S: LayoutScalar>() {
    let f = split_box_fixture::<S>();
    // Exercise the checked Native input and actual completed Layout publication.
    let batch = f.compute();
    assert!(batch.placed_text().is_empty());
    let mut actual = (10..16u32)
        .map(|node| {
            let value = output(&batch, node);
            assert_eq!(value.size, Size::splat(n(10.0)));
            (node - 10, value.location.x, value.location.y)
        })
        .collect::<Vec<_>>();
    actual.sort_by(|a, b| a.1.to_f64().total_cmp(&b.1.to_f64()));
    assert_eq!(
        actual.iter().map(|row| row.0).collect::<Vec<_>>(),
        [0, 1, 2, 5, 4, 3],
        "physical BoxEdge metadata must not fence the resolved atomic content"
    );
    let expected = [0.0, 10.0, 23.0, 53.0, 43.0, 33.0];
    for (index, x) in expected.into_iter().enumerate() {
        assert_eq!(output(&batch, 10 + index as u32).location.x, n(x));
        assert_eq!(output(&batch, 10 + index as u32).location.y, S::ZERO);
    }
    for (node, start, after) in [(20, 20.0, 23.0), (21, 63.0, 68.0)] {
        let value = output(&batch, node);
        assert_eq!(value.location.x, n(start));
        assert_eq!(value.location.x + value.size.width, n(after));
    }
    // Candidate/selected Native fitting must agree with the published geometry.
    let report = layout_mixed_inline_run(f.numeric()).unwrap();
    assert_eq!(report.lines.len(), 1);
    assert_eq!(report.atomics.len(), 6);
    assert_eq!(report.controls.len(), 2);
    assert_eq!(report.inline_extent, n(68.0));
    for row in &report.atomics {
        assert_eq!(row.item.size, Size::splat(n(10.0)));
        let index = row.item.source_index - 10;
        assert_eq!(
            crate::inline::expect_logical_position(row.position).inline,
            n(expected[index])
        );
    }
    for (source, start, after) in [(20, 20.0, 23.0), (21, 63.0, 68.0)] {
        let row = report
            .controls
            .iter()
            .find(|row| row.source_index == source)
            .unwrap();
        let from = crate::inline::expect_logical_position(row.position).inline;
        assert_eq!(from, n(start));
        assert_eq!(from + row.inline_extent, n(after));
        assert_eq!(row.margin_flow, None);
    }
}

#[test]
fn numeric_box_edges_do_not_fence_atomic_order_f32() {
    box_edges_preserve_atomic_order::<f32>();
}
#[test]
fn numeric_box_edges_do_not_fence_atomic_order_f64() {
    box_edges_preserve_atomic_order::<f64>();
}
