use super::*;
#[path = "signed_empty_input.rs"]
mod input;

fn empty_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<usize, S>,
    start: usize,
    end: usize,
    height: f64,
    edges: [(f64, f64, f64); 2],
) {
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let out = |node| {
            entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        assert_eq!(
            out(0).size,
            Size::new(S::from_f64(60.0), S::from_f64(height))
        );
        // Each expected edge is (strip start, nonnegative extent, margin).
        for (rank, node, x, extent, margin) in [
            (
                0,
                start,
                edges[0].0,
                edges[0].1,
                Edges::new(S::ZERO, S::ZERO, S::ZERO, S::from_f64(edges[0].2)),
            ),
            (
                1,
                end,
                edges[1].0,
                edges[1].1,
                Edges::new(S::ZERO, S::from_f64(edges[1].2), S::ZERO, S::ZERO),
            ),
        ] {
            assert_eq!(out(node).source_index.get(), rank);
            assert_eq!(out(node).location.x, S::from_f64(x));
            assert_eq!(
                out(node).location.y,
                S::from_f64(if height == 0.0 { 0.0 } else { 8.0 })
            );
            assert_eq!(out(node).size, Size::new(S::from_f64(extent), S::ZERO));
            assert_eq!(out(node).margin, margin);
        }
    }
}

fn empty<S: LayoutScalar>() {
    for (start_components, end_components, height, start_x, end_x, start_extent, end_extent) in [
        ((5.0, 0.0, 0.0), (-5.0, 0.0, 0.0), 10.0, 5.0, 5.0, 0.0, 0.0),
        ((-5.0, 2.0, 3.0), (0.0, 0.0, 0.0), 10.0, -5.0, 0.0, 5.0, 0.0),
        (
            (-5.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
            10.0,
            -5.0,
            -5.0,
            0.0,
            0.0,
        ),
        ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0, 0.0),
    ] {
        let edge = |(margin, border, padding)| {
            InlineBoundaryEdgeOf::try_new(
                S::from_f64(margin),
                S::from_f64(border),
                S::from_f64(padding),
            )
            .unwrap()
        };
        let (tree, start, end) =
            input::empty_pair::<S>(edge(start_components), edge(end_components));
        let expected = [
            (start_x, start_extent, start_components.0),
            (end_x, end_extent, end_components.0),
        ];
        let prior = tree.compute(60.0).unwrap();
        empty_geometry(&prior, start, end, height, expected);
        // Both endpoints carry the same supplied8/10 witness. Individual
        // nonzero components, not net flow, distinguish a real line from phantom.
        for node in [start, end] {
            let actual = tree.layout_input(node).as_inline_boundary().unwrap();
            assert_eq!(actual.metrics().baseline(), S::from_f64(8.0));
            assert_eq!(actual.metrics().line_extent(), S::from_f64(10.0));
        }
        let retry = tree.compute(60.0).unwrap();
        empty_geometry(&retry, start, end, height, expected);
        empty_geometry(&prior, start, end, height, expected);
    }
}

#[test]
fn signed_empty_pair_components_format_actual_supplied_metrics_even_at_zero_or_negative_flow() {
    empty::<f32>();
    empty::<f64>();
}

fn prefix_pair<S: LayoutScalar>(prefix: S, margin: S) -> (NumericTree<S>, usize, usize, usize) {
    let (mut tree, start, end) =
        input::empty_pair::<S>(InlineBoundaryEdgeOf::ZERO, InlineBoundaryEdgeOf::ZERO);
    let width = prefix + prefix;
    assert!(width.is_finite());
    let mut root = tree.node_input(0).clone();
    root.size.width = PreferredSizeOf::value(LengthPercentageOf::px(width).unwrap());
    tree.inputs[0] = LayoutInputOf::box_input(root);
    let before = tree.boundary(
        InlineBoundaryKind::Start,
        strut(0.0, 0.0, LineBoxAlignment::Baseline),
    );
    tree.children.rotate_right(1);
    assert_eq!(tree.children, [before, start, end]);
    let old = tree.layout_input(before).as_inline_boundary().unwrap();
    tree.inputs[before] = LayoutInputOf::inline_boundary(
        InlineBoundaryInputOf::contribution(
            old.kind(),
            old.metrics(),
            InlineBoundaryEdgeOf::try_new(S::ZERO, prefix, S::ZERO).unwrap(),
        )
        .unwrap(),
    );
    let old = tree.layout_input(start).as_inline_boundary().unwrap();
    tree.inputs[start] = LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
        old.kind(),
        old.metrics(),
        InlineBoundaryEdgeOf::try_new(margin, S::ZERO, S::ZERO).unwrap(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    ));
    assert!(tree.atomics.is_empty());
    (tree, before, start, end)
}

fn prefix_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<usize, S>,
    prefix: S,
    before: usize,
    start: usize,
    end: usize,
) {
    assert!(batch.placed_text().is_empty());
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let out = |node| {
            entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        assert_eq!(out(0).size, Size::new(prefix + prefix, S::from_f64(10.0)));
        assert_eq!(out(before).location.x, S::ZERO);
        assert_eq!(out(before).size.width, prefix);
        assert_eq!(out(before).source_index.get(), 0);
        for (index, node) in [(1, start), (2, end)] {
            assert_eq!(out(node).source_index.get(), index);
            assert_eq!(out(node).location.x, prefix);
            assert_eq!(out(node).location.y, S::from_f64(8.0));
            assert_eq!(out(node).size, Size::new(S::ZERO, S::ZERO));
        }
    }
}

fn recipient_loss<S: LayoutScalar>(prefix: S) {
    let (valid, before, start, end) = prefix_pair(prefix, S::ZERO);
    let prior = valid.compute((prefix + prefix).to_f64()).unwrap();
    prefix_geometry(&prior, prefix, before, start, end);
    let (bad, _, bad_start, _) = prefix_pair(prefix, S::ONE);
    // Components and local flow1 are valid. The failure is applying that actual
    // nonzero margin at the huge incoming coordinate, not source construction.
    let edge = bad
        .layout_input(bad_start)
        .as_inline_boundary()
        .unwrap()
        .edge();
    assert_eq!((edge.margin(), edge.flow_advance()), (S::ONE, S::ONE));
    let error = bad.compute((prefix + prefix).to_f64()).unwrap_err();
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::TextNumeric {
                reason: LayoutTextNumericError::NonZeroLost,
                ..
            }
        ),
        "{error:?}"
    );
    prefix_geometry(&prior, prefix, before, start, end);
    let retry = valid.compute((prefix + prefix).to_f64()).unwrap();
    prefix_geometry(&retry, prefix, before, start, end);
    prefix_geometry(&prior, prefix, before, start, end);
}

#[test]
fn a_valid_margin_must_survive_its_actual_native_recipient_in_each_scalar_lane() {
    recipient_loss::<f32>(16_777_216.0);
    recipient_loss::<f64>(9_007_199_254_740_992.0);
}

#[test]
fn rejecting_a_cancelled_margin_contribution_preserves_the_prior_real_box_result() {
    fn lane<S: LayoutScalar>() {
        let edge = InlineBoundaryEdgeOf::<S>::try_new(
            S::from_f64(-5.0),
            S::from_f64(2.0),
            S::from_f64(3.0),
        )
        .unwrap();
        let (tree, start, end) = input::empty_pair(edge, InlineBoundaryEdgeOf::ZERO);
        let prior = tree.compute(60.0).unwrap();
        empty_geometry(
            &prior,
            start,
            end,
            10.0,
            [(-5.0, 5.0, -5.0), (0.0, 0.0, 0.0)],
        );
        let actual = tree.layout_input(start).as_inline_boundary().unwrap();
        assert_eq!(
            InlineBoundaryInputOf::contribution(actual.kind(), actual.metrics(), actual.edge())
                .unwrap_err(),
            InlineBoundaryEdgeErrorOf::MarginRequiresBoxEdge {
                margin: S::from_f64(-5.0)
            }
        );
        empty_geometry(
            &prior,
            start,
            end,
            10.0,
            [(-5.0, 5.0, -5.0), (0.0, 0.0, 0.0)],
        );
        let retry = tree.compute(60.0).unwrap();
        empty_geometry(
            &retry,
            start,
            end,
            10.0,
            [(-5.0, 5.0, -5.0), (0.0, 0.0, 0.0)],
        );
    }
    lane::<f32>();
    lane::<f64>();
}
