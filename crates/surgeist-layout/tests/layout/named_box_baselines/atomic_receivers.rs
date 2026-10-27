//! Actual atomic provenance supplies a named strut only on a real receiver.
use super::*;

fn receiver_tree<S: l::LayoutScalar>(native_edge: bool, tail: bool) -> Host<S> {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    let central = l::InlineBaselineContext::Named(l::BaselineName::Central);
    let mut host = Host::<S>::new(root(mode, direction, central));
    let mut first = atomic(mode, direction, 10.0, (0.0, 0.0));
    first.vertical_align = l::VerticalAlign::Top;
    first.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
        t::BidiLevel::try_new(0).unwrap(),
        if native_edge {
            l::InlineBreakOpportunity::Prohibited
        } else {
            l::InlineBreakOpportunity::Mandatory
        },
    ));
    if native_edge {
        // This actual leaf box's native exterior is the sole hard cause.
        // No allocated LineBreakInput or fabricated scope supplies its strut.
        first.wrap_edges.after = l::WrapEdge::Line;
    }
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(first)), 1);
    if tail {
        assert_eq!(
            host.push(
                Some(0),
                l::LayoutInputOf::box_input(atomic(mode, direction, 6.0, (0.0, 0.0)))
            ),
            2
        );
    }
    host
}

fn actual_atomic_central_receiver<S: l::LayoutScalar>(native_edge: bool) {
    let mode = l::WritingMode::HorizontalTb;
    // First qualify the real producer without a receiving contributor. The
    // actual10-high Top atom owns only extent; phantom completion cannot demand
    // its absent Central fact or create another formatted/named line.
    let no_tail = receiver_tree::<S>(native_edge, false)
        .compute(mode, &[])
        .unwrap();
    for entries in [no_tail.unrounded_entries(), no_tail.final_entries()] {
        assert_eq!(entries.len(), 2);
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(10.0)));
        assert_eq!(node(entries, 1).location, l::Point::ZERO);
        assert_eq!(node(entries, 1).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(node(entries, 0).source_index, l::SourceIndex::new(0));
        assert_eq!(node(entries, 1).source_index, l::SourceIndex::new(0));
        absent(node(entries, 1).named_baselines);
        let pair = node(entries, 0)
            .named_baselines
            .get(l::BaselineName::Central);
        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
        assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
    }
    assert!(no_tail.placed_text().is_empty());
    assert!(no_tail.final_inline_fragments().is_empty());

    // A fresh complete tree preserves the SAME actual source box and supplies
    // one real baseline-aligned successor. WM4 margin synthesis gives source
    // Central5/5 and successor3/3, so receiver extent10/baseline5. With line1's
    // extent10, global Central15 places the6-high successor at y12.
    // Relabelling raw Physical10/0 would instead give23/20/y17 and is invalid.
    let full = receiver_tree::<S>(native_edge, true)
        .compute(mode, &[])
        .unwrap();
    for entries in [full.unrounded_entries(), full.final_entries()] {
        assert_eq!(entries.len(), 3);
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
        assert_eq!(node(entries, 1).location, l::Point::ZERO);
        assert_eq!(node(entries, 1).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(node(entries, 2).location, l::Point::new(S::ZERO, n(12.0)));
        assert_eq!(node(entries, 2).size, l::Size::new(n(20.0), n(6.0)));
        for (id, source) in [(0, 0), (1, 0), (2, 1)] {
            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
        }
        // Synthesis belongs to the parent receiver, not the child's own table.
        absent(node(entries, 1).named_baselines);
        absent(node(entries, 2).named_baselines);
        let pair = node(entries, 0)
            .named_baselines
            .get(l::BaselineName::Central);
        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
        source_fact(pair.last(), l::PhysicalAxis::Vertical, 15.0);
    }
    assert!(full.placed_text().is_empty());
    assert!(full.final_inline_fragments().is_empty());
}

#[test]
fn actual_atomic_mandatory_receiver_uses_central_margin_baseline_f32() {
    actual_atomic_central_receiver::<f32>(false);
}
#[test]
fn actual_atomic_mandatory_receiver_uses_central_margin_baseline_f64() {
    actual_atomic_central_receiver::<f64>(false);
}
#[test]
fn actual_atomic_native_after_line_receiver_uses_central_margin_baseline_f32() {
    actual_atomic_central_receiver::<f32>(true);
}
#[test]
fn actual_atomic_native_after_line_receiver_uses_central_margin_baseline_f64() {
    actual_atomic_central_receiver::<f64>(true);
}
