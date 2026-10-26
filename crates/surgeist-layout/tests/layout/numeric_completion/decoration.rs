//! Numeric line existence follows retained content and actual endings.
use super::*;

fn allocated_endpoint<S: l::LayoutScalar>(
    direction: l::Direction,
    kind: l::InlineBoundaryKind,
    baseline: f64,
    extent: f64,
    edge: l::InlineBoundaryEdgeOf<S>,
) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::inline_boundary(
        l::InlineBoundaryInputOf::new(
            kind,
            l::InlineMetricsOf::try_new(S::from_f64(baseline), S::from_f64(extent)).unwrap(),
            edge,
        )
        .with_direction(direction),
    )
}
fn pair_with_one_positive_side<S: l::LayoutScalar>(
    direction: l::Direction,
    decorated: l::InlineBoundaryKind,
    border: bool,
    baseline: f64,
    extent: f64,
) -> Vec<l::LayoutInputOf<S>> {
    let positive = if border {
        l::InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::ZERO).unwrap()
    } else {
        l::InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(3.0)).unwrap()
    };
    [l::InlineBoundaryKind::Start, l::InlineBoundaryKind::End]
        .into_iter()
        .map(|kind| {
            allocated_endpoint(
                direction,
                kind,
                baseline,
                extent,
                if kind == decorated {
                    positive
                } else {
                    l::InlineBoundaryEdgeOf::ZERO
                },
            )
        })
        .collect()
}
fn assert_own_edge<S: l::LayoutScalar>(
    actual: l::NodeOutputOf<S>,
    direction: l::Direction,
    kind: l::InlineBoundaryKind,
    border: bool,
) {
    let on_left = matches!(
        (direction, kind),
        (l::Direction::Ltr, l::InlineBoundaryKind::Start)
            | (l::Direction::Rtl, l::InlineBoundaryKind::End)
    );
    let components = |value| {
        if on_left {
            l::Edges {
                left: value,
                ..l::Edges::all(S::ZERO)
            }
        } else {
            l::Edges {
                right: value,
                ..l::Edges::all(S::ZERO)
            }
        }
    };
    assert_eq!(
        actual.border,
        components(if border { S::from_f64(2.0) } else { S::ZERO })
    );
    assert_eq!(
        actual.padding,
        components(if border { S::ZERO } else { S::from_f64(3.0) })
    );
    assert_eq!(
        actual.size,
        l::Size::new(S::from_f64(if border { 2.0 } else { 3.0 }), S::ZERO)
    );
}
fn positive_endpoint<S: l::LayoutScalar>(direction: l::Direction) {
    for decorated in [l::InlineBoundaryKind::Start, l::InlineBoundaryKind::End] {
        for border in [false, true] {
            // This new actual native box owns8/10 at BOTH endpoints. Each
            // case authors exactly one positive component, never a cancelling sum.
            let tree = NumericTree::<S>::new(
                direction,
                pair_with_one_positive_side(direction, decorated, border, 8.0, 10.0),
            );
            let batch = tree.compute();
            assert_numeric_only(&batch);
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(
                    node(entries, 0).size,
                    l::Size::new(S::from_f64(60.0), S::from_f64(10.0))
                );
                let id = if decorated == l::InlineBoundaryKind::Start {
                    1
                } else {
                    2
                };
                let actual = node(entries, id);
                assert_eq!(actual.source_index, l::SourceIndex::new((id - 1) as usize));
                assert_eq!(actual.location.y, S::from_f64(8.0));
                assert_own_edge(actual, direction, decorated, border);
                let other = node(entries, if id == 1 { 2 } else { 1 });
                assert_eq!(other.border, l::Edges::all(S::ZERO));
                assert_eq!(other.padding, l::Edges::all(S::ZERO));
                assert_eq!(other.size, l::Size::ZERO);
                assert_eq!(other.location.y, S::from_f64(8.0));
            }
        }
    }
}
#[test]
fn each_positive_start_or_end_border_or_padding_keeps_numeric_line_real() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        positive_endpoint::<f32>(direction);
        positive_endpoint::<f64>(direction);
    }
}
fn adjoining_blocks<S: l::LayoutScalar>(
    direction: l::Direction,
) -> (l::LayoutInputOf<S>, l::LayoutInputOf<S>) {
    let block = |height, top, bottom| {
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            direction,
            size: l::Size::new(
                l::PreferredSizeOf::AUTO,
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(height)).unwrap()),
            ),
            margin: l::Edges {
                top: l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(top)).unwrap()),
                bottom: l::LengthAutoOf::value(
                    l::LengthPercentageOf::px(S::from_f64(bottom)).unwrap(),
                ),
                ..l::Edges::all(l::LengthAutoOf::ZERO)
            },
            ..l::NodeInputOf::default()
        })
    };
    (block(5.0, 0.0, 10.0), block(7.0, 20.0, 0.0))
}
fn assert_separated_blocks<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    b_id: u32,
    line_height: f64,
) {
    assert_eq!(
        node(entries, 0).size,
        l::Size::new(S::from_f64(60.0), S::from_f64(42.0 + line_height))
    );
    let a = node(entries, 1);
    let b = node(entries, b_id);
    assert_eq!(a.location, l::Point::ZERO);
    assert_eq!(a.size, l::Size::new(S::from_f64(60.0), S::from_f64(5.0)));
    assert_eq!(
        b.location,
        l::Point::new(S::ZERO, S::from_f64(35.0 + line_height))
    );
    assert_eq!(b.size, l::Size::new(S::from_f64(60.0), S::from_f64(7.0)));
    assert_eq!(a.source_index, l::SourceIndex::new(0));
    assert_eq!(b.source_index, l::SourceIndex::new((b_id - 1) as usize));
}
fn zero_atomic_barrier<S: l::LayoutScalar>(direction: l::Direction) {
    let (a, b) = adjoining_blocks::<S>(direction);
    // No endpoint/default strut is present: this actual zero-size in-flow
    // atomic's true numerical baseline and line extent are both0.
    let tree = NumericTree::<S>::new(direction, vec![a, atomic(direction, 0.0, 0.0), b]);
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_separated_blocks(entries, 3, 0.0);
        let actual = node(entries, 2);
        assert_eq!(actual.size, l::Size::ZERO);
        assert_eq!(actual.source_index, l::SourceIndex::new(1));
        assert_eq!(
            actual.location,
            l::Point::new(
                S::from_f64(if direction == l::Direction::Ltr {
                    0.0
                } else {
                    60.0
                }),
                S::from_f64(15.0)
            )
        );
    }
}
#[test]
fn true_zero_height_in_flow_atomic_line_stops_block_margin_collapse() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        zero_atomic_barrier::<f32>(direction);
        zero_atomic_barrier::<f64>(direction);
    }
}
fn zero_metric_decoration_barrier<S: l::LayoutScalar>(direction: l::Direction) {
    for decorated in [l::InlineBoundaryKind::Start, l::InlineBoundaryKind::End] {
        for border in [false, true] {
            let (a, b) = adjoining_blocks::<S>(direction);
            // This separate box genuinely owns0/0 metrics, not an erased
            // allocation from the8/10 fixture. Its own BP component is positive.
            let mut participants = vec![a];
            participants.extend(pair_with_one_positive_side(
                direction, decorated, border, 0.0, 0.0,
            ));
            participants.push(b);
            let tree = NumericTree::<S>::new(direction, participants);
            let batch = tree.compute();
            assert_numeric_only(&batch);
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_separated_blocks(entries, 4, 0.0);
                let id = if decorated == l::InlineBoundaryKind::Start {
                    2
                } else {
                    3
                };
                let actual = node(entries, id);
                assert_eq!(actual.source_index, l::SourceIndex::new((id - 1) as usize));
                assert_eq!(actual.location.y, S::from_f64(15.0));
                assert_own_edge(actual, direction, decorated, border);
            }
        }
    }
}
#[test]
fn positive_individual_bp_with_true_zero_metrics_stops_block_margin_collapse() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        zero_metric_decoration_barrier::<f32>(direction);
        zero_metric_decoration_barrier::<f64>(direction);
    }
}
fn forced_margin_barrier<S: l::LayoutScalar>(direction: l::Direction) {
    let (a, b) = adjoining_blocks::<S>(direction);
    let tree = NumericTree::<S>::new(direction, vec![a, forced(direction), b]);
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        //5+10+real Forced20+20 gives B.y55 and root height62. Its terminal
        // phantom receiver must not add any further used line extent.
        assert_separated_blocks(entries, 3, 20.0);
        let actual = node(entries, 2);
        assert_eq!(actual.source_index, l::SourceIndex::new(1));
        assert_eq!(actual.location.y, S::from_f64(31.0));
    }
}
#[test]
fn actual_forced_line_stops_collapsing_margins_without_terminal_phantom_extent() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        forced_margin_barrier::<f32>(direction);
        forced_margin_barrier::<f64>(direction);
    }
}

fn zero_forced_margin_barrier<S: l::LayoutScalar>(direction: l::Direction) {
    let (a, b) = adjoining_blocks::<S>(direction);
    // These are the actual supplied0/0 metrics of a genuine Forced control;
    // ending semantics, not positive extent, make this a real zero-height line.
    let control = l::LayoutInputOf::line_break(
        l::LineBreakInputOf::new()
            .with_direction(direction)
            .with_metrics(l::InlineMetricsOf::try_new(S::ZERO, S::ZERO).unwrap()),
    );
    let tree = NumericTree::<S>::new(direction, vec![a, control, b]);
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_separated_blocks(entries, 3, 0.0);
        let actual = node(entries, 2);
        assert_eq!(actual.source_index, l::SourceIndex::new(1));
        assert_eq!(actual.location.y, S::from_f64(15.0));
    }
}
#[test]
fn actual_zero_metric_forced_control_still_stops_block_margin_collapse() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        zero_forced_margin_barrier::<f32>(direction);
        zero_forced_margin_barrier::<f64>(direction);
    }
}
