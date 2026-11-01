//! Atomic Top/Bottom aligns actual margin edges in physical line directions.
use super::*;

fn single_signed_top<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    let mut host = Host::<S>::new(root(mode, direction, l::InlineBaselineContext::Physical));
    let mut item = atomic(mode, direction, 10.0, (-2.0, 6.0));
    item.vertical_align = l::VerticalAlignOf::Top;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(item)), 1);
    let batch = host.compute(mode, &[]).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), 2);
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(14.0)));
        assert_eq!(node(entries, 1).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(node(entries, 1).margin.top, n(-2.0));
        assert_eq!(node(entries, 1).margin.bottom, n(6.0));
        assert_eq!(node(entries, 0).source_index, l::SourceIndex::new(0));
        assert_eq!(node(entries, 1).source_index, l::SourceIndex::new(0));
        absent(node(entries, 1).named_baselines);
        // Atomic bounds are its real margin box. Its14-unit span starts0;
        // border top−2 makes margin-over −2−(−2)=0 and margin-under14.
        assert_eq!(node(entries, 1).location, l::Point::new(S::ZERO, n(-2.0)));
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

fn vertical_lr_taller_peer<S: l::LayoutScalar>(alignment: l::VerticalAlignOf<S>, expected_x: f64) {
    let mode = l::WritingMode::VerticalLr;
    let direction = l::Direction::Ltr;
    let mut host = Host::<S>::new(root(mode, direction, l::InlineBaselineContext::Physical));
    let mut item = atomic(mode, direction, 10.0, (0.0, 0.0));
    item.vertical_align = alignment;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(item)), 1);
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(mode, direction, 30.0, (0.0, 0.0)))
        ),
        2
    );
    let batch = host.compute(mode, &[]).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), 3);
        assert_eq!(node(entries, 0).size, l::Size::new(n(30.0), n(100.0)));
        assert_eq!(node(entries, 2).location, l::Point::new(S::ZERO, n(20.0)));
        assert_eq!(node(entries, 2).size, l::Size::new(n(30.0), n(20.0)));
        assert_eq!(node(entries, 1).size, l::Size::new(n(10.0), n(20.0)));
        assert_eq!(node(entries, 1).margin, l::Edges::ZERO);
        assert_eq!(node(entries, 2).margin, l::Edges::ZERO);
        for (id, source) in [(0, 0), (1, 0), (2, 1)] {
            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
        }
        absent(node(entries, 1).named_baselines);
        absent(node(entries, 2).named_baselines);
        // Real level0 Ltr source order gives A y0, peer y20. The peer's30
        // dominates the10-high group. WM4 line-over is RIGHT (A x20 for
        // Top), line-under LEFT (A x0 for Bottom), regardless of block-start.
        assert_eq!(
            node(entries, 1).location,
            l::Point::new(n(expected_x), S::ZERO)
        );
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

#[test]
fn atomic_top_signed_margin_aligns_actual_over_edge_f32() {
    single_signed_top::<f32>();
}
#[test]
fn atomic_top_signed_margin_aligns_actual_over_edge_f64() {
    single_signed_top::<f64>();
}
#[test]
fn vertical_lr_atomic_top_aligns_line_over_with_taller_peer_f32() {
    vertical_lr_taller_peer::<f32>(l::VerticalAlignOf::Top, 20.0);
}
#[test]
fn vertical_lr_atomic_top_aligns_line_over_with_taller_peer_f64() {
    vertical_lr_taller_peer::<f64>(l::VerticalAlignOf::Top, 20.0);
}
#[test]
fn vertical_lr_atomic_bottom_aligns_line_under_with_taller_peer_f32() {
    vertical_lr_taller_peer::<f32>(l::VerticalAlignOf::Bottom, 0.0);
}
#[test]
fn vertical_lr_atomic_bottom_aligns_line_under_with_taller_peer_f64() {
    vertical_lr_taller_peer::<f64>(l::VerticalAlignOf::Bottom, 0.0);
}

#[derive(Clone, Copy, Debug)]
struct EdgeSpec<S: l::LayoutScalar> {
    mode: l::WritingMode,
    direction: l::Direction,
    context: l::InlineBaselineContext,
    alignment: l::VerticalAlignOf<S>,
    margins: (f64, f64),
    peer: bool,
}
fn edge_tree<S: l::LayoutScalar>(spec: EdgeSpec<S>) -> Host<S> {
    let mut host = Host::<S>::new(root(spec.mode, spec.direction, spec.context));
    let mut item = atomic(spec.mode, spec.direction, 10.0, spec.margins);
    item.vertical_align = spec.alignment;
    host.push(Some(0), l::LayoutInputOf::box_input(item));
    if spec.peer {
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(spec.mode, spec.direction, 30.0, (0.0, 0.0))),
        );
    }
    host
}
fn edge_point<S: l::LayoutScalar>(mode: l::WritingMode, inline: f64, cross: f64) -> l::Point<S> {
    // Expand physical coordinate notation only; no production axis helper
    // chooses expected sides, margin signs, or scalar baseline values.
    if mode == l::WritingMode::HorizontalTb {
        l::Point::new(n(inline), n(cross))
    } else {
        l::Point::new(n(cross), n(inline))
    }
}
fn edge_geometry_matrix<S: l::LayoutScalar>() {
    for mode in [
        l::WritingMode::HorizontalTb,
        l::WritingMode::VerticalRl,
        l::WritingMode::VerticalLr,
        l::WritingMode::SidewaysRl,
        l::WritingMode::SidewaysLr,
    ] {
        let over_right = matches!(
            mode,
            l::WritingMode::VerticalRl | l::WritingMode::VerticalLr | l::WritingMode::SidewaysRl
        );
        for direction in [l::Direction::Ltr, l::Direction::Rtl] {
            for context in [
                l::InlineBaselineContext::Physical,
                l::InlineBaselineContext::Named(l::BaselineName::Alphabetic),
                l::InlineBaselineContext::Named(l::BaselineName::Central),
            ] {
                // Margins, single span, single forward/right-over border,
                // taller-peer forward Top/Bottom, taller-peer right Top/Bottom.
                // The rows are literal Box3/WM4 edge derivations. Negative
                // totals still have positive span4, with real border overflow.
                for (index, (margins, span, single, forward, right)) in [
                    ((0.0, 0.0), 10.0, [0.0, 0.0], [0.0, 20.0], [20.0, 0.0]),
                    ((-2.0, 6.0), 14.0, [-2.0, 6.0], [-2.0, 14.0], [22.0, 6.0]),
                    ((3.0, 5.0), 18.0, [3.0, 5.0], [3.0, 15.0], [17.0, 5.0]),
                    ((6.0, -2.0), 14.0, [6.0, -2.0], [6.0, 22.0], [14.0, -2.0]),
                    ((-2.0, -4.0), 4.0, [-2.0, -4.0], [-2.0, 24.0], [22.0, -4.0]),
                ]
                .into_iter()
                .enumerate()
                {
                    // Full Physical geometry matrix; selected signed/positive
                    // Named contrasts exercise different baseline-group origins
                    // without needlessly repeating every margin combination.
                    if context != l::InlineBaselineContext::Physical && index != 1 && index != 2 {
                        continue;
                    }
                    for peer in [false, true] {
                        for alignment in [l::VerticalAlignOf::Top, l::VerticalAlignOf::Bottom] {
                            let spec = EdgeSpec {
                                mode,
                                direction,
                                context,
                                alignment,
                                margins,
                                peer,
                            };
                            let batch = edge_tree::<S>(spec)
                                .compute(mode, &[])
                                .unwrap_or_else(|error| panic!("{spec:?}: {error:?}"));
                            let cross = if peer { 30.0 } else { span };
                            let expected_cross = if peer {
                                let values = if over_right { right } else { forward };
                                values[usize::from(alignment == l::VerticalAlignOf::Bottom)]
                            } else {
                                single[usize::from(over_right)]
                            };
                            let (item_inline, peer_inline) = match (mode, direction) {
                                (l::WritingMode::SidewaysLr, l::Direction::Ltr) => (80.0, 60.0),
                                (l::WritingMode::SidewaysLr, l::Direction::Rtl) => (0.0, 20.0),
                                (_, l::Direction::Ltr) => (0.0, 20.0),
                                (_, l::Direction::Rtl) => (80.0, 60.0),
                            };
                            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                                assert_eq!(entries.len(), if peer { 3 } else { 2 });
                                assert_eq!(
                                    node(entries, 0).size,
                                    if mode == l::WritingMode::HorizontalTb {
                                        l::Size::new(n(100.0), n(cross))
                                    } else {
                                        l::Size::new(n(cross), n(100.0))
                                    },
                                    "{spec:?}"
                                );
                                assert_eq!(
                                    node(entries, 1).location,
                                    edge_point(mode, item_inline, expected_cross),
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
                                let expected_margin = if mode == l::WritingMode::HorizontalTb {
                                    l::Edges {
                                        top: n(margins.0),
                                        bottom: n(margins.1),
                                        ..l::Edges::ZERO
                                    }
                                } else if over_right {
                                    l::Edges {
                                        right: n(margins.0),
                                        left: n(margins.1),
                                        ..l::Edges::ZERO
                                    }
                                } else {
                                    l::Edges {
                                        left: n(margins.0),
                                        right: n(margins.1),
                                        ..l::Edges::ZERO
                                    }
                                };
                                assert_eq!(node(entries, 1).margin, expected_margin);
                                assert_eq!(node(entries, 0).source_index, l::SourceIndex::new(0));
                                assert_eq!(node(entries, 1).source_index, l::SourceIndex::new(0));
                                absent(node(entries, 1).named_baselines);
                                if peer {
                                    assert_eq!(
                                        node(entries, 2).location,
                                        edge_point(mode, peer_inline, 0.0),
                                        "{spec:?}"
                                    );
                                    assert_eq!(
                                        node(entries, 2).size,
                                        if mode == l::WritingMode::HorizontalTb {
                                            l::Size::new(n(20.0), n(30.0))
                                        } else {
                                            l::Size::new(n(30.0), n(20.0))
                                        }
                                    );
                                    assert_eq!(
                                        node(entries, 2).source_index,
                                        l::SourceIndex::new(1)
                                    );
                                    absent(node(entries, 2).named_baselines);
                                }
                                for name in [l::BaselineName::Alphabetic, l::BaselineName::Central]
                                {
                                    let pair = node(entries, 0).named_baselines.get(name);
                                    if peer && context == l::InlineBaselineContext::Named(name) {
                                        let anchor = if name == l::BaselineName::Central {
                                            15.0
                                        } else if matches!(
                                            mode,
                                            l::WritingMode::HorizontalTb
                                                | l::WritingMode::SidewaysLr
                                        ) {
                                            30.0
                                        } else {
                                            0.0
                                        };
                                        named(
                                            node(entries, 0).named_baselines,
                                            name,
                                            if mode == l::WritingMode::HorizontalTb {
                                                l::PhysicalAxis::Vertical
                                            } else {
                                                l::PhysicalAxis::Horizontal
                                            },
                                            anchor,
                                        );
                                    } else {
                                        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                                        assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
                                    }
                                }
                            }
                            assert!(batch.placed_text().is_empty());
                            assert!(batch.final_inline_fragments().is_empty());
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn atomic_margin_edges_have_literal_geometry_across_modes_and_directions_f32() {
    edge_geometry_matrix::<f32>();
}
#[test]
fn atomic_margin_edges_have_literal_geometry_across_modes_and_directions_f64() {
    edge_geometry_matrix::<f64>();
}

fn edge_cache_and_publication<S: l::LayoutScalar>() {
    for mode in [
        l::WritingMode::HorizontalTb,
        l::WritingMode::VerticalRl,
        l::WritingMode::VerticalLr,
        l::WritingMode::SidewaysRl,
        l::WritingMode::SidewaysLr,
    ] {
        let right = matches!(
            mode,
            l::WritingMode::VerticalRl | l::WritingMode::VerticalLr | l::WritingMode::SidewaysRl
        );
        let before = EdgeSpec {
            mode,
            direction: l::Direction::Ltr,
            context: l::InlineBaselineContext::Physical,
            alignment: l::VerticalAlignOf::Top,
            margins: (-2.0, 6.0),
            peer: true,
        };
        let mut host = edge_tree::<S>(before);
        let cold = host.compute(mode, &[]).unwrap();
        let old = semantic(&cold);
        cold.apply_to(&mut host).unwrap();
        let hits: usize = host.hits.iter().map(Cell::get).sum();
        let warm = host.compute(mode, &[]).unwrap();
        assert_eq!(semantic(&warm), old);
        assert_eq!(warm.final_entries(), cold.final_entries());
        assert!(host.hits.iter().map(Cell::get).sum::<usize>() > hits);
        warm.apply_to(&mut host).unwrap();
        let after = EdgeSpec {
            context: l::InlineBaselineContext::Named(l::BaselineName::Central),
            alignment: l::VerticalAlignOf::Bottom,
            margins: (3.0, 5.0),
            ..before
        };
        host.change(0, |input| input.inline_baseline_context = after.context);
        let changed_input = edge_tree::<S>(after).inputs[1].as_box().unwrap().clone();
        host.inputs[1] = l::LayoutInputOf::box_input(changed_input);
        let changed = host.compute(mode, &[0, 1]).unwrap();
        let fresh = edge_tree::<S>(after).compute(mode, &[]).unwrap();
        assert_eq!(semantic(&changed), semantic(&fresh));
        assert_eq!(changed.final_entries(), fresh.final_entries());
        for entries in [changed.unrounded_entries(), changed.final_entries()] {
            assert_eq!(
                node(entries, 1).location,
                edge_point(
                    mode,
                    if mode == l::WritingMode::SidewaysLr {
                        80.0
                    } else {
                        0.0
                    },
                    if right { 5.0 } else { 15.0 }
                )
            );
            named(
                node(entries, 0).named_baselines,
                l::BaselineName::Central,
                if mode == l::WritingMode::HorizontalTb {
                    l::PhysicalAxis::Vertical
                } else {
                    l::PhysicalAxis::Horizontal
                },
                15.0,
            );
        }
        let accepted = semantic(&changed);
        changed.apply_to(&mut host).unwrap();
        let counts = (host.prepares.get(), host.commits);
        // Genuine missing resolved participation must refuse despite warm box
        // geometry; no cached result can authorize an unadmitted atomic role.
        host.change(1, |input| input.atomic_inline_participation = None);
        let error = host.compute(mode, &[1]).unwrap_err();
        assert!(
            matches!(
                error.kind(),
                l::LayoutErrorKindOf::InvalidInput(
                    l::LayoutInvalidInputOf::AtomicInlineParticipation {
                        reason: l::AtomicInlineParticipationRoleError::MissingForAtomicInline
                    }
                )
            ),
            "{error:?}"
        );
        assert_eq!((host.prepares.get(), host.commits), counts);
        assert_eq!(semantic(host.published.batch.as_ref().unwrap()), accepted);
        assert_eq!(semantic(&cold), old);
        assert_eq!(semantic(&changed), accepted);
        host.change(1, |input| {
            input.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
                t::BidiLevel::try_new(0).unwrap(),
                l::InlineBreakOpportunity::Allowed,
            ))
        });
        let restored = host.compute(mode, &[1]).unwrap();
        assert_eq!(semantic(&restored), semantic(&fresh));
    }
}
#[test]
fn atomic_margin_edge_changes_use_real_warm_cache_and_preserve_prior_packets_f32() {
    edge_cache_and_publication::<f32>();
}
#[test]
fn atomic_margin_edge_changes_use_real_warm_cache_and_preserve_prior_packets_f64() {
    edge_cache_and_publication::<f64>();
}

fn explicit_control_offsets<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for context in [
        l::InlineBaselineContext::Physical,
        l::InlineBaselineContext::Named(l::BaselineName::Alphabetic),
        l::InlineBaselineContext::Named(l::BaselineName::Central),
    ] {
        for (alignment, y) in [
            (l::VerticalAlignOf::Top, 2.0),
            (l::VerticalAlignOf::Bottom, 22.0),
        ] {
            let mut host = Host::<S>::new(root(mode, l::Direction::Ltr, context));
            let metrics =
                l::InlineMetricsOf::try_new(l::InlineBaselineContext::Physical, n(2.0), n(10.0))
                    .unwrap();
            for kind in [l::InlineBoundaryKind::Start, l::InlineBoundaryKind::End] {
                host.push(
                    Some(0),
                    l::LayoutInputOf::inline_boundary(
                        l::InlineBoundaryInputOf::contribution(
                            kind,
                            metrics,
                            l::InlineBoundaryEdgeOf::ZERO,
                        )
                        .with_vertical_align(alignment),
                    ),
                );
            }
            host.push(
                Some(0),
                l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, 30.0, (0.0, 0.0))),
            );
            let batch = host.compute(mode, &[]).unwrap();
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(entries.len(), 4);
                assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(30.0)));
                assert_eq!(node(entries, 3).location, l::Point::ZERO);
                assert_eq!(node(entries, 3).size, l::Size::new(n(20.0), n(30.0)));
                for (id, index) in [(0, 0), (1, 0), (2, 1), (3, 2)] {
                    assert_eq!(node(entries, id).source_index, l::SourceIndex::new(index));
                }
                // Control outputs are allocated baseline points, not atomic
                // border boxes: Top baseline2; Bottom30−after8=22.
                for id in [1, 2] {
                    assert_eq!(node(entries, id).location, l::Point::new(S::ZERO, n(y)));
                    assert_eq!(node(entries, id).size, l::Size::ZERO);
                }
            }
        }
    }
}
#[test]
fn atomic_edge_fix_preserves_explicit_control_strut_offsets_f32() {
    explicit_control_offsets::<f32>();
}
#[test]
fn atomic_edge_fix_preserves_explicit_control_strut_offsets_f64() {
    explicit_control_offsets::<f64>();
}
