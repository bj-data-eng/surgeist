use super::*;

fn intrinsic_fold_batch<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    available: AvailableOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let input = LayoutFlowInput::try_new(tree, 0, Vec::new(), wrapping(true), Vec::new()).unwrap();
    compute_layout_with_flow(
        &input,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn intrinsic_fold_geometry<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, margin: S) {
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    let ten = S::from_f64(10.0);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let out = |node| {
            entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        // The actual normalized body and executed endpoint positions precede
        // the intrinsic assertion; raw negative geometry is never replaced.
        assert_eq!(out(3).location, Point::new(-margin, S::ZERO));
        assert_eq!(out(3).size, Size::new(ten, ten));
        assert_eq!(out(3).margin, Edges::ZERO);
        for (node, x) in [(1, S::ZERO), (4, -margin), (2, ten - margin), (5, ten)] {
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
        // One unbreakable actual piece executes0 -> -M -> -M+10 ->10.
        // Numeric CSS intrinsic demand is10, independently of its raw overhang.
        assert_eq!(out(0).size, Size::new(ten, ten));
    }
}

fn intrinsic_fold_lane<S: LayoutScalar>(margin: S, available: AvailableOf<S>) {
    let ten = S::from_f64(10.0);
    assert_ne!(margin + ten, margin);
    assert_ne!(-margin + ten, -margin);
    assert_eq!((margin + ten) - margin, S::from_f64(8.0));
    assert_eq!((-margin + ten) + margin, ten);
    let mut tree = fold_tree(margin, false);
    let mut root = tree.node_input(0).clone();
    root.size.width = PreferredSizeOf::px(ten);
    tree = tree.style(0, root);
    let prior = intrinsic_fold_batch(&tree, AvailableOf::MAX_CONTENT);
    intrinsic_fold_geometry(&prior, margin);
    let mut root = tree.node_input(0).clone();
    root.size.width = PreferredSizeOf::AUTO;
    tree = tree.style(0, root);
    assert_eq!(tree.node_input(0).size.width, PreferredSizeOf::AUTO);
    let actual = intrinsic_fold_batch(&tree, available);
    intrinsic_fold_geometry(&actual, margin);
    let retry = intrinsic_fold_batch(&tree, available);
    intrinsic_fold_geometry(&retry, margin);
    intrinsic_fold_geometry(&actual, margin);
    intrinsic_fold_geometry(&prior, margin);
}

#[test]
fn min_content_uses_actual_reordered_signed_native_flow_f32() {
    intrinsic_fold_lane::<f32>(33_554_432.0, AvailableOf::MIN_CONTENT);
}

#[test]
fn min_content_uses_actual_reordered_signed_native_flow_f64() {
    intrinsic_fold_lane::<f64>(18_014_398_509_481_984.0, AvailableOf::MIN_CONTENT);
}

#[test]
fn max_content_uses_actual_reordered_signed_native_flow_f32() {
    intrinsic_fold_lane::<f32>(33_554_432.0, AvailableOf::MAX_CONTENT);
}

#[test]
fn max_content_uses_actual_reordered_signed_native_flow_f64() {
    intrinsic_fold_lane::<f64>(18_014_398_509_481_984.0, AvailableOf::MAX_CONTENT);
}
