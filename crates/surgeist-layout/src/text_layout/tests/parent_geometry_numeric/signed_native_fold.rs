use super::*;

fn fold_tree<S: LayoutScalar>(margin: S, end_aligned: bool) -> PublicLayoutTreeOf<S> {
    let frame = FRAMES[0];
    let mut tree = fixture::<S>(frame, true);
    let mut root = tree.node_input(0).clone();
    root.size.width = PreferredSizeOf::px(S::from_f64(16.0));
    root.text_align = if end_aligned {
        TextAlign::End
    } else {
        TextAlign::Start
    };
    root.text_align_last = if end_aligned {
        TextAlignLast::End
    } else {
        TextAlignLast::Start
    };
    tree = tree.style(0, root);
    let formatter = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let child_parent = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    for (node, kind, value, parent) in [
        (1, InlineBoundaryKind::Start, S::ZERO, formatter),
        (2, InlineBoundaryKind::Start, margin, child_parent),
        (4, InlineBoundaryKind::End, -margin, child_parent),
        (5, InlineBoundaryKind::End, S::ZERO, formatter),
    ] {
        let previous = tree.layout_input(node).as_inline_boundary().unwrap();
        assert_eq!(previous.kind(), kind);
        let edge = InlineBoundaryEdgeOf::try_new(value, S::ZERO, S::ZERO).unwrap();
        assert_eq!(edge.flow_advance(), value);
        assert_eq!(edge.border_padding_advance(), S::ZERO);
        let input = InlineBoundaryInputOf::box_edge(kind, previous.metrics(), edge, parent)
            .with_writing_mode(previous.writing_mode())
            .with_direction(previous.direction())
            .with_vertical_align(previous.vertical_align());
        assert_eq!(input.parent_edge_axes(), Some(parent));
        assert_eq!(
            (input.writing_mode(), input.direction()),
            (WritingMode::HorizontalTb, Direction::Ltr)
        );
        tree.insert_input(node, LayoutInputOf::inline_boundary(input));
    }
    // Preserve the maintained fixture's actual body, levels and break policy.
    let body = tree.node_input(3);
    assert_eq!(
        body.size,
        Size::new(
            PreferredSizeOf::px(S::from_f64(10.0)),
            PreferredSizeOf::px(S::from_f64(10.0)),
        )
    );
    assert_eq!(
        body.atomic_inline_participation,
        Some(AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            InlineBreakOpportunity::Prohibited,
        ))
    );
    assert_eq!(body.margin, Edges::all(LengthAutoOf::ZERO));
    tree
}

fn fold_batch<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> CompletedLayoutBatchOf<u32, S> {
    let input = LayoutFlowInput::try_new(tree, 0, Vec::new(), wrapping(true), Vec::new()).unwrap();
    compute_layout_with_flow(
        &input,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(16.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn fold_geometry<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, margin: S, start: f64) {
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    let start = S::from_f64(start);
    let end = start + S::from_f64(10.0);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let out = |node| {
            entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        // Fixed containing width is not an intrinsic-width or opaque-fit oracle.
        assert_eq!(out(0).size, Size::new(S::from_f64(16.0), S::from_f64(10.0)));
        assert_eq!(out(3).location, Point::new(start - margin, S::ZERO));
        assert_eq!(out(3).size, Size::splat(S::from_f64(10.0)));
        assert_eq!(out(3).margin, Edges::ZERO);
        // Actual canonical Pstart, Cend, body, Cstart, Pend execution.
        for (node, x) in [(1, start), (4, start - margin), (2, end - margin), (5, end)] {
            assert_eq!(out(node).location, Point::new(x, S::ZERO));
            assert_eq!(out(node).size, Size::ZERO);
            assert_eq!(out(node).border, Edges::ZERO);
            assert_eq!(out(node).padding, Edges::ZERO);
        }
        assert_eq!(out(1).margin, Edges::ZERO);
        assert_eq!(out(5).margin, Edges::ZERO);
        assert_eq!(out(2).margin, Side::Right.edges(margin.to_f64()));
        assert_eq!(out(4).margin, Side::Left.edges(-margin.to_f64()));
        for (rank, node) in [1, 2, 3, 4, 5].into_iter().enumerate() {
            assert_eq!(out(node).source_index.get(), rank);
        }
    }
}

fn fold_lane<S: LayoutScalar>(margin: S) {
    let ten = S::from_f64(10.0);
    // These exactly chosen native values expose nonassociativity, without
    // wholly absorbing the real body or requiring a new precision policy.
    assert_ne!(margin + ten, margin);
    assert_ne!(-margin + ten, -margin);
    assert_eq!((margin + ten) - margin, S::from_f64(8.0));
    assert_eq!((-margin + ten) + margin, ten);
    let ordinary = fold_tree(margin, false);
    let prior = fold_batch(&ordinary);
    fold_geometry(&prior, margin, 0.0);
    let aligned = fold_tree(margin, true);
    let actual = fold_batch(&aligned);
    // Used physical flow10 in a16px End-aligned frame starts at6. The real
    // atomic remains at6-M..16-M; no raw occupied rectangle is moved to0..10.
    fold_geometry(&actual, margin, 6.0);
    let retry = fold_batch(&aligned);
    fold_geometry(&retry, margin, 6.0);
    fold_geometry(&actual, margin, 6.0);
    fold_geometry(&prior, margin, 0.0);
}

#[test]
fn end_alignment_uses_actual_reordered_signed_native_flow_f32() {
    fold_lane::<f32>(33_554_432.0);
}

#[test]
fn end_alignment_uses_actual_reordered_signed_native_flow_f64() {
    fold_lane::<f64>(18_014_398_509_481_984.0);
}

#[path = "signed_native_fold_intrinsic.rs"]
mod intrinsic;
