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

#[derive(Clone, Copy, Debug)]
struct ReceiverSpec {
    mode: l::WritingMode,
    direction: l::Direction,
    name: l::BaselineName,
    alignment: l::VerticalAlign,
    native_edge: bool,
    signed: bool,
}
fn matrix_tree<S: l::LayoutScalar>(spec: ReceiverSpec, tail: bool) -> Host<S> {
    let mut host = Host::<S>::new(root(
        spec.mode,
        spec.direction,
        l::InlineBaselineContext::Named(spec.name),
    ));
    let mut first = atomic(
        spec.mode,
        spec.direction,
        10.0,
        if spec.signed { (-2.0, 6.0) } else { (0.0, 0.0) },
    );
    first.vertical_align = spec.alignment;
    first.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
        t::BidiLevel::try_new(u8::from(spec.direction == l::Direction::Rtl)).unwrap(),
        if spec.native_edge {
            l::InlineBreakOpportunity::Prohibited
        } else {
            l::InlineBreakOpportunity::Mandatory
        },
    ));
    if spec.native_edge {
        first.wrap_edges.after = l::WrapEdge::Line;
    }
    host.push(Some(0), l::LayoutInputOf::box_input(first));
    if tail {
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(spec.mode, spec.direction, 6.0, (0.0, 0.0))),
        );
    }
    host
}
fn literal_point<S: l::LayoutScalar>(mode: l::WritingMode, inline: f64, cross: f64) -> l::Point<S> {
    // Only the literal table's physical-coordinate notation is expanded here;
    // expected sides/anchors never come from production FlowAxes.
    if mode == l::WritingMode::HorizontalTb {
        l::Point::new(n(inline), n(cross))
    } else {
        l::Point::new(n(cross), n(inline))
    }
}
fn empty_receiver_matrix<S: l::LayoutScalar>() {
    // Physical cross: full A, tailAlpha/Central, lastAlpha/Central, no-tail A.
    // These literal rows follow WM4 Table11 plus the actual signed margin box.
    for (mode, plain, signed) in [
        (
            l::WritingMode::HorizontalTb,
            [0.0, 14.0, 12.0, 20.0, 15.0, 0.0],
            [-2.0, 22.0, 18.0, 28.0, 21.0, -2.0],
        ),
        (
            l::WritingMode::VerticalRl,
            [10.0, 0.0, 2.0, 0.0, 5.0, 0.0],
            [20.0, 0.0, 4.0, 0.0, 7.0, 6.0],
        ),
        (
            l::WritingMode::VerticalLr,
            [0.0, 10.0, 12.0, 10.0, 15.0, 0.0],
            [6.0, 14.0, 18.0, 14.0, 21.0, 6.0],
        ),
        (
            l::WritingMode::SidewaysRl,
            [10.0, 0.0, 2.0, 0.0, 5.0, 0.0],
            [20.0, 0.0, 4.0, 0.0, 7.0, 6.0],
        ),
        (
            l::WritingMode::SidewaysLr,
            [0.0, 14.0, 12.0, 20.0, 15.0, 0.0],
            [-2.0, 22.0, 18.0, 28.0, 21.0, -2.0],
        ),
    ] {
        for direction in [l::Direction::Ltr, l::Direction::Rtl] {
            let inline = match (mode, direction) {
                (l::WritingMode::SidewaysLr, l::Direction::Ltr) => 80.0,
                (l::WritingMode::SidewaysLr, l::Direction::Rtl) => 0.0,
                (_, l::Direction::Ltr) => 0.0,
                (_, l::Direction::Rtl) => 80.0,
            };
            for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
                for alignment in [l::VerticalAlign::Top, l::VerticalAlign::Bottom] {
                    for native_edge in [false, true] {
                        for signed_margins in [false, true] {
                            let spec = ReceiverSpec {
                                mode,
                                direction,
                                name,
                                alignment,
                                native_edge,
                                signed: signed_margins,
                            };
                            let values = if signed_margins { signed } else { plain };
                            let extent = if signed_margins { 14.0 } else { 10.0 };
                            let preserve_signed_top =
                                signed_margins && alignment == l::VerticalAlign::Top;
                            let physical_control = |tail| {
                                let mut control = matrix_tree::<S>(spec, tail);
                                control.change(0, |input| {
                                    input.inline_baseline_context =
                                        l::InlineBaselineContext::Physical
                                });
                                control.compute(mode, &[]).unwrap()
                            };
                            let no_tail = matrix_tree::<S>(spec, false)
                                .compute(mode, &[])
                                .unwrap_or_else(|error| panic!("no-tail {spec:?}: {error:?}"));
                            for entries in [no_tail.unrounded_entries(), no_tail.final_entries()] {
                                assert_eq!(entries.len(), 2);
                                assert_eq!(
                                    node(entries, 0).size,
                                    if mode == l::WritingMode::HorizontalTb {
                                        l::Size::new(n(100.0), n(extent))
                                    } else {
                                        l::Size::new(n(extent), n(100.0))
                                    }
                                );
                                if !preserve_signed_top {
                                    assert_eq!(
                                        node(entries, 1).location,
                                        literal_point(mode, inline, values[5]),
                                        "{spec:?}"
                                    );
                                }
                                let pair = node(entries, 0).named_baselines.get(name);
                                assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                                assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
                                absent(node(entries, 1).named_baselines);
                            }
                            if preserve_signed_top {
                                // This extension preserves original Top geometry.
                                // Its preexisting Physical margin-sign defect is
                                // separate; do not adopt that value as CSS truth.
                                let control = physical_control(false);
                                for (entries, physical) in
                                    [no_tail.unrounded_entries(), no_tail.final_entries()]
                                        .into_iter()
                                        .zip([control.unrounded_entries(), control.final_entries()])
                                {
                                    assert_eq!(
                                        node(physical, 0).size,
                                        if mode == l::WritingMode::HorizontalTb {
                                            l::Size::new(n(100.0), n(14.0))
                                        } else {
                                            l::Size::new(n(14.0), n(100.0))
                                        }
                                    );
                                    assert_eq!(
                                        node(entries, 1).location,
                                        node(physical, 1).location,
                                        "preserved original-line geometry {spec:?}"
                                    );
                                }
                            }
                            let full = matrix_tree::<S>(spec, true)
                                .compute(mode, &[])
                                .unwrap_or_else(|error| panic!("full {spec:?}: {error:?}"));
                            let (tail_cross, anchor) = if name == l::BaselineName::Alphabetic {
                                (values[1], values[3])
                            } else {
                                (values[2], values[4])
                            };
                            for entries in [full.unrounded_entries(), full.final_entries()] {
                                assert_eq!(entries.len(), 3);
                                assert_eq!(
                                    node(entries, 0).size,
                                    if mode == l::WritingMode::HorizontalTb {
                                        l::Size::new(n(100.0), n(extent * 2.0))
                                    } else {
                                        l::Size::new(n(extent * 2.0), n(100.0))
                                    },
                                    "{spec:?}"
                                );
                                if !preserve_signed_top {
                                    assert_eq!(
                                        node(entries, 1).location,
                                        literal_point(mode, inline, values[0]),
                                        "{spec:?}"
                                    );
                                }
                                assert_eq!(
                                    node(entries, 2).location,
                                    literal_point(mode, inline, tail_cross),
                                    "{spec:?}"
                                );
                                assert_eq!(
                                    node(entries, 1).size,
                                    if mode == l::WritingMode::HorizontalTb {
                                        l::Size::new(n(20.0), n(10.0))
                                    } else {
                                        l::Size::new(n(10.0), n(20.0))
                                    }
                                );
                                assert_eq!(
                                    node(entries, 2).size,
                                    if mode == l::WritingMode::HorizontalTb {
                                        l::Size::new(n(20.0), n(6.0))
                                    } else {
                                        l::Size::new(n(6.0), n(20.0))
                                    }
                                );
                                for (id, source) in [(0, 0), (1, 0), (2, 1)] {
                                    assert_eq!(
                                        node(entries, id).source_index,
                                        l::SourceIndex::new(source)
                                    );
                                }
                                absent(node(entries, 1).named_baselines);
                                absent(node(entries, 2).named_baselines);
                                let pair = node(entries, 0).named_baselines.get(name);
                                assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                                source_fact(
                                    pair.last(),
                                    if mode == l::WritingMode::HorizontalTb {
                                        l::PhysicalAxis::Vertical
                                    } else {
                                        l::PhysicalAxis::Horizontal
                                    },
                                    anchor,
                                );
                            }
                            if preserve_signed_top {
                                let control = physical_control(true);
                                for (entries, physical) in
                                    [full.unrounded_entries(), full.final_entries()]
                                        .into_iter()
                                        .zip([control.unrounded_entries(), control.final_entries()])
                                {
                                    assert_eq!(
                                        node(physical, 0).size,
                                        if mode == l::WritingMode::HorizontalTb {
                                            l::Size::new(n(100.0), n(28.0))
                                        } else {
                                            l::Size::new(n(28.0), n(100.0))
                                        }
                                    );
                                    assert_eq!(
                                        node(entries, 1).location,
                                        node(physical, 1).location,
                                        "preserved original-line geometry {spec:?}"
                                    );
                                }
                            }
                            assert!(full.placed_text().is_empty());
                            assert!(full.final_inline_fragments().is_empty());
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn atomic_receivers_use_literal_named_margin_geometry_all_modes_f32() {
    empty_receiver_matrix::<f32>();
}
#[test]
fn atomic_receivers_use_literal_named_margin_geometry_all_modes_f64() {
    empty_receiver_matrix::<f64>();
}

fn receiver_incoming_authority<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for native_edge in [false, true] {
        for (alignment, y) in [
            (l::VerticalAlign::Top, 10.0),
            (l::VerticalAlign::Bottom, 14.0),
        ] {
            let mut host = receiver_tree::<S>(native_edge, true);
            host.change(2, |input| input.vertical_align = alignment);
            let batch = host.compute(mode, &[]).unwrap();
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
                assert_eq!(node(entries, 1).location, l::Point::ZERO);
                assert_eq!(node(entries, 2).location, l::Point::new(S::ZERO, n(y)));
                absent(node(entries, 1).named_baselines);
                absent(node(entries, 2).named_baselines);
                // T contributes only Top/Bottom extent6. Incoming source A's
                // resolved5/5 is the ONLY authority for receiving Central15.
                let pair = node(entries, 0)
                    .named_baselines
                    .get(l::BaselineName::Central);
                assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                source_fact(pair.last(), l::PhysicalAxis::Vertical, 15.0);
            }
        }
    }
}
#[test]
fn incoming_atomic_receiver_is_the_only_named_anchor_f32() {
    receiver_incoming_authority::<f32>();
}
#[test]
fn incoming_atomic_receiver_is_the_only_named_anchor_f64() {
    receiver_incoming_authority::<f64>();
}

fn available_tree<S: l::LayoutScalar>(
    spec: ReceiverSpec,
    preference: l::BaselineSource,
    tail: bool,
) -> Host<S> {
    let mut host = matrix_tree::<S>(spec, tail);
    host.change(1, |input| {
        input.inline_baseline_context = l::InlineBaselineContext::Named(spec.name);
        input.baseline_source = preference;
    });
    // Genuine ordinary Block lines establish first2/last8. The Central
    // producer has real12-unit content in its fixed10 border; no font is used.
    let heights = if spec.name == l::BaselineName::Alphabetic {
        (2.0, 6.0)
    } else {
        (4.0, 8.0)
    };
    source_line(&mut host, 1, spec.name, heights.0);
    source_line(&mut host, 1, spec.name, heights.1);
    host
}
fn receiver_available_preferences<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for (name, preference, height, last, tail_y) in [
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::First,
            24.0,
            16.0,
            10.0,
        ),
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::Last,
            20.0,
            18.0,
            12.0,
        ),
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::Auto,
            20.0,
            18.0,
            12.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::First,
            21.0,
            13.0,
            10.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::Last,
            21.0,
            18.0,
            15.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::Auto,
            21.0,
            18.0,
            15.0,
        ),
    ] {
        for alignment in [l::VerticalAlign::Top, l::VerticalAlign::Bottom] {
            for native_edge in [false, true] {
                let spec = ReceiverSpec {
                    mode,
                    direction: l::Direction::Ltr,
                    name,
                    alignment,
                    native_edge,
                    signed: false,
                };
                let no_tail = available_tree::<S>(spec, preference, false)
                    .compute(mode, &[])
                    .unwrap();
                for entries in [no_tail.unrounded_entries(), no_tail.final_entries()] {
                    assert_eq!(node(entries, 0).size.height, n(10.0));
                    let own = node(entries, 1).named_baselines.get(name);
                    source_fact(own.first(), l::PhysicalAxis::Vertical, 2.0);
                    source_fact(own.last(), l::PhysicalAxis::Vertical, 8.0);
                    let root_pair = node(entries, 0).named_baselines.get(name);
                    assert_eq!(root_pair.first(), l::BaselineFactOf::Unknown);
                    assert_eq!(root_pair.last(), l::BaselineFactOf::Unknown);
                }
                let full = available_tree::<S>(spec, preference, true)
                    .compute(mode, &[])
                    .unwrap();
                for entries in [full.unrounded_entries(), full.final_entries()] {
                    assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(height)));
                    assert_eq!(node(entries, 1).location, l::Point::ZERO);
                    assert_eq!(node(entries, 1).size, l::Size::new(n(20.0), n(10.0)));
                    assert_eq!(node(entries, 2).location, l::Point::new(S::ZERO, n(tail_y)));
                    let own = node(entries, 1).named_baselines.get(name);
                    source_fact(own.first(), l::PhysicalAxis::Vertical, 2.0);
                    source_fact(own.last(), l::PhysicalAxis::Vertical, 8.0);
                    let root_pair = node(entries, 0).named_baselines.get(name);
                    assert_eq!(root_pair.first(), l::BaselineFactOf::Unknown);
                    source_fact(root_pair.last(), l::PhysicalAxis::Vertical, last);
                    for (id, index) in [(0, 0), (1, 0), (2, 1), (3, 0), (4, 0), (5, 1), (6, 0)] {
                        assert_eq!(node(entries, id).source_index, l::SourceIndex::new(index));
                    }
                    assert_eq!(entries.len(), 7);
                }
            }
        }
    }
    // Selected available border-relative8 plus signed margins gives6/8;
    // it is neither absent-under synthesis nor a double addition of margins.
    let spec = ReceiverSpec {
        mode,
        direction: l::Direction::Ltr,
        name: l::BaselineName::Alphabetic,
        alignment: l::VerticalAlign::Bottom,
        native_edge: true,
        signed: true,
    };
    let mut host = available_tree::<S>(spec, l::BaselineSource::Last, true);
    // A's own dominant name differs; its actual ordinary children still export
    // the requested Alpha facts. The receiver must not substitute A's own name.
    host.change(1, |input| {
        input.inline_baseline_context = l::InlineBaselineContext::Named(l::BaselineName::Central)
    });
    let batch = host.compute(mode, &[]).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 0).size.height, n(28.0));
        assert_eq!(node(entries, 1).location.y, n(-2.0));
        assert_eq!(node(entries, 2).location.y, n(14.0));
        let pair = node(entries, 0)
            .named_baselines
            .get(l::BaselineName::Alphabetic);
        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
        source_fact(pair.last(), l::PhysicalAxis::Vertical, 20.0);
    }
}
#[test]
fn atomic_receiver_retains_real_nested_name_source_preference_f32() {
    receiver_available_preferences::<f32>();
}
#[test]
fn atomic_receiver_retains_real_nested_name_source_preference_f64() {
    receiver_available_preferences::<f64>();
}

fn barrier_tree<S: l::LayoutScalar>(
    spec: ReceiverSpec,
    preference: l::BaselineSource,
    unknown_first: bool,
    tail: bool,
) -> Host<S> {
    let mut host = matrix_tree::<S>(spec, tail);
    host.change(1, |input| {
        input.size.height = px(20.0);
        input.inline_baseline_context = l::InlineBaselineContext::Named(spec.name);
        input.baseline_source = preference;
    });
    let top_only = |host: &mut Host<S>| {
        let child = host.push(
            Some(1),
            l::LayoutInputOf::box_input(source_block(spec.name, None)),
        );
        host.push(
            Some(child),
            l::LayoutInputOf::line_break(
                l::LineBreakInputOf::new()
                    .with_metrics(
                        l::InlineMetricsOf::try_new(
                            l::InlineBaselineContext::Physical,
                            n(8.0),
                            n(10.0),
                        )
                        .unwrap(),
                    )
                    .with_vertical_align(l::VerticalAlign::Top),
            ),
        );
    };
    if unknown_first {
        top_only(&mut host);
        source_line(&mut host, 1, spec.name, 6.0);
    } else {
        source_line(&mut host, 1, spec.name, 6.0);
        top_only(&mut host);
    }
    host
}
fn receiver_source_barriers<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        for native_edge in [false, true] {
            let spec = ReceiverSpec {
                mode,
                direction: l::Direction::Ltr,
                name,
                alignment: l::VerticalAlign::Top,
                native_edge,
                signed: false,
            };
            for unknown_first in [false, true] {
                for preference in [
                    l::BaselineSource::Auto,
                    l::BaselineSource::First,
                    l::BaselineSource::Last,
                ] {
                    let no_tail = barrier_tree::<S>(spec, preference, unknown_first, false)
                        .compute(mode, &[])
                        .unwrap();
                    for entries in [no_tail.unrounded_entries(), no_tail.final_entries()] {
                        assert_eq!(node(entries, 0).size.height, n(20.0));
                        let own = node(entries, 1).named_baselines.get(name);
                        let q = if name == l::BaselineName::Alphabetic {
                            6.0
                        } else {
                            3.0
                        };
                        if unknown_first {
                            assert_eq!(own.first(), l::BaselineFactOf::Unknown);
                            source_fact(own.last(), l::PhysicalAxis::Vertical, 10.0 + q);
                        } else {
                            source_fact(own.first(), l::PhysicalAxis::Vertical, q);
                            assert_eq!(own.last(), l::BaselineFactOf::Unknown);
                        }
                        let pair = node(entries, 0).named_baselines.get(name);
                        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                        assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
                    }
                    let selects_first = preference == l::BaselineSource::First;
                    let mut full_host = barrier_tree::<S>(spec, preference, unknown_first, true);
                    if selects_first == unknown_first {
                        // First publish the actual supported opposite preference;
                        // the refused mutation may not replace any completed packet.
                        full_host.change(1, |input| {
                            input.baseline_source = if unknown_first {
                                l::BaselineSource::Last
                            } else {
                                l::BaselineSource::First
                            }
                        });
                        let prior = full_host.compute(mode, &[]).unwrap();
                        let snapshot = semantic(&prior);
                        prior.apply_to(&mut full_host).unwrap();
                        let counts = (full_host.prepares.get(), full_host.commits);
                        full_host.change(1, |input| input.baseline_source = preference);
                        missing(full_host.compute(mode, &[1]));
                        assert_eq!((full_host.prepares.get(), full_host.commits), counts);
                        assert_eq!(
                            semantic(full_host.published.batch.as_ref().unwrap()),
                            snapshot
                        );
                        assert_eq!(semantic(&prior), snapshot);
                    } else {
                        let batch = full_host.compute(mode, &[]).unwrap();
                        let (last, y) = match (name, unknown_first) {
                            (l::BaselineName::Alphabetic, true) => (36.0, 30.0),
                            (l::BaselineName::Alphabetic, false) => (26.0, 20.0),
                            (l::BaselineName::Central, true) => (33.0, 30.0),
                            (l::BaselineName::Central, false) => (23.0, 20.0),
                        };
                        for entries in [batch.unrounded_entries(), batch.final_entries()] {
                            assert_eq!(node(entries, 0).size.height, n(40.0));
                            assert_eq!(node(entries, 2).location.y, n(y));
                            let pair = node(entries, 0).named_baselines.get(name);
                            assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                            source_fact(pair.last(), l::PhysicalAxis::Vertical, last);
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn atomic_receiver_selected_unknown_never_borrows_opposite_set_f32() {
    receiver_source_barriers::<f32>();
}
#[test]
fn atomic_receiver_selected_unknown_never_borrows_opposite_set_f64() {
    receiver_source_barriers::<f64>();
}

#[derive(Clone, Copy, Debug)]
enum RefusedSource {
    Unknown,
    OtherName,
    Orthogonal,
}
fn refused_tree<S: l::LayoutScalar>(spec: ReceiverSpec, kind: RefusedSource) -> Host<S> {
    let mut host = matrix_tree::<S>(spec, false);
    match kind {
        RefusedSource::Unknown => {
            host.push(
                Some(1),
                l::LayoutInputOf::box_input(l::NodeInputOf {
                    display: l::Display::Grid,
                    size: l::Size::new(px(10.0), px(10.0)),
                    ..l::NodeInputOf::default()
                }),
            );
        }
        RefusedSource::OtherName => {
            let other = if spec.name == l::BaselineName::Alphabetic {
                l::BaselineName::Central
            } else {
                l::BaselineName::Alphabetic
            };
            host.change(1, |input| {
                input.inline_baseline_context = l::InlineBaselineContext::Named(other)
            });
            host.push(
                Some(1),
                l::LayoutInputOf::box_input(atomic(spec.mode, spec.direction, 10.0, (0.0, 0.0))),
            );
        }
        RefusedSource::Orthogonal => {
            // A has actual physical20x10 geometry but owns a vertical-rl
            // formatting context with a real6x10 child: AlphaX14/CentralX17.
            host.change(1, |input| {
                input.writing_mode = l::WritingMode::VerticalRl;
                input.inline_baseline_context = l::InlineBaselineContext::Named(spec.name);
            });
            let mut inner = atomic(
                l::WritingMode::VerticalRl,
                l::Direction::Ltr,
                6.0,
                (0.0, 0.0),
            );
            inner.size.height = px(10.0);
            host.push(Some(1), l::LayoutInputOf::box_input(inner));
        }
    }
    host
}
fn receiver_refusals_and_lazy_completion<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        for alignment in [l::VerticalAlign::Top, l::VerticalAlign::Bottom] {
            for native_edge in [false, true] {
                let spec = ReceiverSpec {
                    mode,
                    direction: l::Direction::Ltr,
                    name,
                    alignment,
                    native_edge,
                    signed: false,
                };
                for kind in [
                    RefusedSource::Unknown,
                    RefusedSource::OtherName,
                    RefusedSource::Orthogonal,
                ] {
                    let mut host = refused_tree::<S>(spec, kind);
                    let prior = host
                        .compute(mode, &[])
                        .unwrap_or_else(|error| panic!("no-tail {spec:?} {kind:?}: {error:?}"));
                    for entries in [prior.unrounded_entries(), prior.final_entries()] {
                        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(10.0)));
                        assert_eq!(node(entries, 1).location, l::Point::ZERO);
                        assert_eq!(node(entries, 1).size, l::Size::new(n(20.0), n(10.0)));
                        let root_pair = node(entries, 0).named_baselines.get(name);
                        assert_eq!(root_pair.first(), l::BaselineFactOf::Unknown);
                        assert_eq!(root_pair.last(), l::BaselineFactOf::Unknown);
                        let own = node(entries, 1).named_baselines.get(name);
                        if matches!(kind, RefusedSource::Orthogonal) {
                            for fact in [own.first(), own.last()] {
                                source_fact(
                                    fact,
                                    l::PhysicalAxis::Horizontal,
                                    if name == l::BaselineName::Alphabetic {
                                        14.0
                                    } else {
                                        17.0
                                    },
                                );
                            }
                        } else {
                            assert_eq!(own.first(), l::BaselineFactOf::Unknown);
                            assert_eq!(own.last(), l::BaselineFactOf::Unknown);
                        }
                        if matches!(kind, RefusedSource::OtherName) {
                            let other = if name == l::BaselineName::Alphabetic {
                                l::BaselineName::Central
                            } else {
                                l::BaselineName::Alphabetic
                            };
                            named(
                                node(entries, 1).named_baselines,
                                other,
                                l::PhysicalAxis::Vertical,
                                if other == l::BaselineName::Alphabetic {
                                    10.0
                                } else {
                                    5.0
                                },
                            );
                        }
                    }
                    let snapshot = semantic(&prior);
                    prior.apply_to(&mut host).unwrap();
                    let counts = (host.prepares.get(), host.commits);
                    // Topology changes only by a real receiving successor. The
                    // unchanged source's previously unused facts are now needed.
                    host.push(
                        Some(0),
                        l::LayoutInputOf::box_input(atomic(
                            mode,
                            l::Direction::Ltr,
                            6.0,
                            (0.0, 0.0),
                        )),
                    );
                    let error = host.compute(mode, &[0]).unwrap_err();
                    if matches!(kind, RefusedSource::Orthogonal) {
                        assert!(
                            matches!(
                                error.kind(),
                                l::LayoutErrorKindOf::InvalidInput(
                                    l::LayoutInvalidInputOf::InlineBaseline(
                                        l::InlineBaselineError::AxisMismatch {
                                            expected: l::PhysicalAxis::Vertical,
                                            actual: l::PhysicalAxis::Horizontal
                                        }
                                    )
                                )
                            ),
                            "{error:?}"
                        );
                    } else {
                        assert!(
                            matches!(
                                error.kind(),
                                l::LayoutErrorKindOf::MissingContext(
                                    l::LayoutMissingContext::NamedInlineBaseline
                                )
                            ),
                            "{error:?}"
                        );
                    }
                    assert_eq!((host.prepares.get(), host.commits), counts);
                    assert_eq!(semantic(host.published.batch.as_ref().unwrap()), snapshot);
                    assert_eq!(semantic(&prior), snapshot);
                }
            }
            let reference = l::InlineBaselineContext::Named(name);
            let mut control_host = Host::<S>::new(root(mode, l::Direction::Ltr, reference));
            control_host.push(
                Some(0),
                l::LayoutInputOf::line_break(
                    l::LineBreakInputOf::new()
                        .with_metrics(
                            l::InlineMetricsOf::try_new(
                                l::InlineBaselineContext::Physical,
                                n(8.0),
                                n(10.0),
                            )
                            .unwrap(),
                        )
                        .with_vertical_align(alignment),
                ),
            );
            let no_tail = control_host.compute(mode, &[]).unwrap();
            assert_eq!(node(no_tail.unrounded_entries(), 0).size.height, n(10.0));
            control_host.push(
                Some(0),
                l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, 6.0, (0.0, 0.0))),
            );
            let error = control_host.compute(mode, &[]).unwrap_err();
            // Explicit controls have no atomic box provenance to requalify.
            assert!(
                matches!(error.kind(), l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::InlineBaseline(l::InlineBaselineError::ReferenceMismatch { expected, actual })) if *expected == reference && *actual == l::InlineBaselineContext::Physical),
                "{error:?}"
            );
        }
    }
}
#[test]
fn atomic_receiver_refuses_unknown_other_name_axis_and_keeps_control_refs_f32() {
    receiver_refusals_and_lazy_completion::<f32>();
}
#[test]
fn atomic_receiver_refuses_unknown_other_name_axis_and_keeps_control_refs_f64() {
    receiver_refusals_and_lazy_completion::<f64>();
}

fn sizing_receiver_tree<S: l::LayoutScalar>(name: l::BaselineName, native_edge: bool) -> Host<S> {
    let mode = l::WritingMode::HorizontalTb;
    let reference = l::InlineBaselineContext::Named(name);
    let mut outside = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
    outside.display = l::Display::Flex;
    outside.align_items = Some(l::AlignItems::Start);
    let mut host = Host::<S>::new(outside);
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(root(mode, l::Direction::Ltr, reference))
        ),
        1
    );
    let mut source = atomic(mode, l::Direction::Ltr, 20.0, (0.0, 0.0));
    source.vertical_align = l::VerticalAlign::Top;
    source.inline_baseline_context = reference;
    source.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
        t::BidiLevel::try_new(0).unwrap(),
        if native_edge {
            l::InlineBreakOpportunity::Prohibited
        } else {
            l::InlineBreakOpportunity::Mandatory
        },
    ));
    if native_edge {
        source.wrap_edges.after = l::WrapEdge::Line;
    }
    assert_eq!(host.push(Some(1), l::LayoutInputOf::box_input(source)), 2);
    assert_eq!(
        host.push(
            Some(1),
            l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, 6.0, (0.0, 0.0)))
        ),
        3
    );
    // A real zero Forced owns Available0 despite the fixed20-high source box.
    // Undemanded known-size queries may be Unknown; a deferred real receiver
    // needs this actual0, not raw physical under20 or manufactured absence.
    assert_eq!(
        host.push(
            Some(2),
            l::LayoutInputOf::line_break(
                l::LineBreakInputOf::new().with_metrics(
                    l::InlineMetricsOf::try_new(reference, S::ZERO, S::ZERO).unwrap()
                )
            )
        ),
        4
    );
    host
}
fn receiver_sizing_cache<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for native_edge in [false, true] {
        for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
            let mut host = sizing_receiver_tree::<S>(name, native_edge);
            let cold = host.compute(mode, &[]).unwrap();
            let (height, last) = if name == l::BaselineName::Alphabetic {
                (46.0, 26.0)
            } else {
                (43.0, 23.0)
            };
            for entries in [cold.unrounded_entries(), cold.final_entries()] {
                assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(height)));
                assert_eq!(node(entries, 1).size, l::Size::new(n(100.0), n(height)));
                assert_eq!(node(entries, 1).location, l::Point::ZERO);
                assert_eq!(node(entries, 2).size, l::Size::new(n(20.0), n(20.0)));
                assert_eq!(node(entries, 2).location, l::Point::ZERO);
                assert_eq!(node(entries, 3).location, l::Point::new(S::ZERO, n(20.0)));
                named(
                    node(entries, 2).named_baselines,
                    name,
                    l::PhysicalAxis::Vertical,
                    0.0,
                );
                let pair = node(entries, 1).named_baselines.get(name);
                assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                source_fact(pair.last(), l::PhysicalAxis::Vertical, last);
                for (id, index) in [(0, 0), (1, 0), (2, 0), (3, 1), (4, 0)] {
                    assert_eq!(node(entries, id).source_index, l::SourceIndex::new(index));
                }
                assert_eq!(entries.len(), 5);
            }
            let mut unknown_keys = Vec::new();
            let mut available_keys = Vec::new();
            for row in cold
                .cache_store_entries()
                .iter()
                .filter(|row| row.node() == 2)
            {
                let pair = row.output().named_baselines.get(name);
                for fact in [pair.first(), pair.last()] {
                    match fact {
                        l::BaselineFactOf::Unknown => {}
                        l::BaselineFactOf::Available(_) => {
                            source_fact(fact, l::PhysicalAxis::Vertical, 0.0)
                        }
                        l::BaselineFactOf::Absent => panic!(
                            "actual zero Forced is not absent; request={:?}",
                            row.input()
                        ),
                    }
                }
                let key = (*row.input(), row.context());
                let keys = if matches!(pair.last(), l::BaselineFactOf::Available(_)) {
                    &mut available_keys
                } else {
                    &mut unknown_keys
                };
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
            eprintln!(
                "atomic receiver sizing scalar={} name={name:?} native_edge={native_edge} unknown_keys={} available_keys={}",
                std::any::type_name::<S>(),
                unknown_keys.len(),
                available_keys.len()
            );
            let snapshot = semantic(&cold);
            cold.apply_to(&mut host).unwrap();
            let hits: usize = host.hits.iter().map(Cell::get).sum();
            let warm = host.compute(mode, &[]).unwrap();
            assert_eq!(semantic(&warm), snapshot);
            assert_eq!(warm.final_entries(), cold.final_entries());
            assert!(
                host.hits.iter().map(Cell::get).sum::<usize>() > hits,
                "the real size/child host cache must actually answer a warm request"
            );
            warm.apply_to(&mut host).unwrap();
            // A genuine style/name change recomputes the same topology and
            // compares to a fresh public host, then returns to the old name.
            let other = if name == l::BaselineName::Alphabetic {
                l::BaselineName::Central
            } else {
                l::BaselineName::Alphabetic
            };
            host.change(1, |input| {
                input.inline_baseline_context = l::InlineBaselineContext::Named(other)
            });
            // Source A owns only the old requested name: the new real receiver
            // must refuse, retaining the accepted packet rather than borrowing it.
            let counts = (host.prepares.get(), host.commits);
            missing(host.compute(mode, &[1]));
            assert_eq!((host.prepares.get(), host.commits), counts);
            assert_eq!(semantic(host.published.batch.as_ref().unwrap()), snapshot);
            host.change(1, |input| {
                input.inline_baseline_context = l::InlineBaselineContext::Named(name)
            });
            let restored = host.compute(mode, &[1]).unwrap();
            let fresh = sizing_receiver_tree::<S>(name, native_edge)
                .compute(mode, &[])
                .unwrap();
            assert_eq!(semantic(&restored), semantic(&fresh));
            assert_eq!(semantic(&cold), snapshot);
        }
    }
}
#[test]
fn deferred_atomic_receiver_qualifies_size_only_source_and_warm_cache_f32() {
    receiver_sizing_cache::<f32>();
}
#[test]
fn deferred_atomic_receiver_qualifies_size_only_source_and_warm_cache_f64() {
    receiver_sizing_cache::<f64>();
}

fn receiver_balance_replay<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        for native_edge in [false, true] {
            for style in [l::TextWrapStyle::Auto, l::TextWrapStyle::Balance] {
                let spec = ReceiverSpec {
                    mode,
                    direction: l::Direction::Ltr,
                    name,
                    alignment: l::VerticalAlign::Top,
                    native_edge,
                    signed: false,
                };
                let mut host = matrix_tree::<S>(spec, true);
                host.change(0, |input| input.text_wrap_style = style);
                host.change(2, |input| input.size.width = px(60.0));
                for _ in 0..2 {
                    let mut next = atomic(mode, l::Direction::Ltr, 6.0, (0.0, 0.0));
                    next.size.width = px(40.0);
                    host.push(Some(0), l::LayoutInputOf::box_input(next));
                }
                let batch = host.compute(mode, &[]).unwrap();
                let (alpha, last, first_y, second_y) = if name == l::BaselineName::Alphabetic {
                    (true, 26.0, 14.0, 20.0)
                } else {
                    (false, 23.0, 12.0, 20.0)
                };
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    assert_eq!(entries.len(), 5);
                    assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(26.0)));
                    assert_eq!(node(entries, 1).location, l::Point::ZERO);
                    assert_eq!(
                        node(entries, 2).location,
                        l::Point::new(S::ZERO, n(first_y))
                    );
                    // The actual Forced group is frozen. For remaining widths
                    //60/40/40, Auto's100/40 variance900 improves uniquely to
                    // Balance's60/80 variance100 with the SAME2 receiving rows.
                    let (ux, uy, vx) = if style == l::TextWrapStyle::Auto {
                        (60.0, first_y, 0.0)
                    } else {
                        (0.0, second_y, 40.0)
                    };
                    assert_eq!(node(entries, 3).location, l::Point::new(n(ux), n(uy)));
                    assert_eq!(node(entries, 4).location, l::Point::new(n(vx), n(second_y)));
                    for (id, index) in [(0, 0), (1, 0), (2, 1), (3, 2), (4, 3)] {
                        assert_eq!(node(entries, id).source_index, l::SourceIndex::new(index));
                    }
                    let pair = node(entries, 0).named_baselines.get(if alpha {
                        l::BaselineName::Alphabetic
                    } else {
                        l::BaselineName::Central
                    });
                    assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                    source_fact(pair.last(), l::PhysicalAxis::Vertical, last);
                }
                let snapshot = semantic(&batch);
                batch.apply_to(&mut host).unwrap();
                let repeated = host.compute(mode, &[]).unwrap();
                assert_eq!(semantic(&repeated), snapshot);
                assert_eq!(repeated.final_entries(), batch.final_entries());
                assert_eq!(semantic(&batch), snapshot);
            }
        }
    }
}
#[test]
fn atomic_receiver_named_metrics_survive_real_balance_group_replay_f32() {
    receiver_balance_replay::<f32>();
}
#[test]
fn atomic_receiver_named_metrics_survive_real_balance_group_replay_f64() {
    receiver_balance_replay::<f64>();
}

mod float_feedback;
