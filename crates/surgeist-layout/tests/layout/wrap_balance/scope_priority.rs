//! Native scope avoidance precedes Balance's preference among legal numeric cuts.
mod stable;

use super::*;

fn fixture<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    style: l::text_flow::TextWrapStyle,
) -> Tree<S> {
    let atomic = |width| {
        l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
            display: l::box_model::Display::InlineBlock,
            direction,
            vertical_align: l::inline::VerticalAlignOf::Baseline,
            atomic_inline_participation: Some(l::inline::AtomicInlineParticipation::new(
                t::supplied_line::BidiLevel::try_new(if direction == l::geometry::Direction::Ltr {
                    0
                } else {
                    1
                })
                .unwrap(),
                l::text_flow::InlineBreakOpportunity::Allowed,
            )),
            size: l::geometry::Size::new(
                l::sizing::PreferredSizeOf::value(
                    l::length::LengthPercentageOf::px(S::from_f64(width)).unwrap(),
                ),
                l::sizing::PreferredSizeOf::value(
                    l::length::LengthPercentageOf::px(S::from_f64(10.0)).unwrap(),
                ),
            ),
            ..l::node::NodeInputOf::default()
        })
    };
    let boundary = |kind| {
        l::node::LayoutInputOf::inline_boundary(
            l::inline::InlineBoundaryInputOf::contribution(
                kind,
                l::inline::InlineMetricsOf::try_new(
                    surgeist_layout::baseline::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                l::inline::InlineBoundaryEdgeOf::ZERO,
            )
            .unwrap()
            .with_direction(direction),
        )
    };
    Tree {
        inputs: vec![
            l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
                display: l::box_model::Display::Block,
                direction,
                text_align: l::text_flow::TextAlign::Start,
                text_align_last: l::text_flow::TextAlignLast::Start,
                text_wrap_style: style,
                size: l::geometry::Size::new(
                    l::sizing::PreferredSizeOf::value(
                        l::length::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                    ),
                    l::sizing::PreferredSizeOf::AUTO,
                ),
                ..l::node::NodeInputOf::default()
            }),
            boundary(l::inline::InlineBoundaryKind::Start),
            atomic(30.0),
            atomic(20.0),
            boundary(l::inline::InlineBoundaryKind::End),
            atomic(10.0),
            atomic(10.0),
        ],
        non_box: l::node::NodeInputOf::non_box(),
        children: vec![
            vec![1, 2, 3, 4, 5, 6],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ],
    }
}

fn compute<S: l::scalar::LayoutScalar>(
    tree: &Tree<S>,
    inside: l::inline_wrapping::WrapInside,
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    let flow = l::text_layout::LayoutFlowInput::try_new(
        tree,
        0,
        Vec::new(),
        l::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: vec![l::inline_wrapping::WrappingScopeInput::NativePair {
                start: 1,
                end: 4,
                wrap_inside: inside,
                edges: l::inline_wrapping::WrapEdges::AUTO,
                // The actual pair has these same Baseline metrics. The witness
                // identifies that contribution; it must not duplicate it.
                metric_contribution: Some(l::inline_wrapping::InlineBoxMetricContribution::Strut(
                    t::allocation::LineBoxStrut {
                        scope: None,
                        metrics: t::allocation::LineStrut {
                            baseline: 8.0,
                            line_extent: 10.0,
                        },
                        alignment: t::allocation::LineBoxAlignment::Baseline,
                    },
                )),
            }],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let request = {
        let sizing_request = crate::layout::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::tree::LayoutTree::node_input(tree, 0).writing_mode,
                surgeist_layout::tree::LayoutTree::node_input(tree, 0).direction,
            ),
            sizing_request
                .orthogonal_fallback()
                .initial_containing_block(),
        )
    };
    let batch = l::compute_layout_with_flow(&flow, 0, request).unwrap();
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    batch
}

fn node<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    id: u32,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, id, crate::layout::observation::Kind::Box)
}
fn node_of_kind<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    id: u32,
    expected_kind: crate::layout::observation::Kind,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, id, expected_kind)
}

fn qualify<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    // Real source order is Start, A30, B20, End, C10, D10. Every
    // atomic supplies its own following Allowed cut; neither endpoint does.
    // Auto takes ABC / D = 60/10. Balance can improve to A / BCD = 30/40:
    // squared deviations from the two-line mean35 are 50 rather than1250.
    // Cuts after A and B are inside the actual pair. The after-C cut is
    // outside it, so its empty violation set strictly dominates either inside
    // cut when Avoid is requested, before Balance compares line lengths.
    for style in [
        l::text_flow::TextWrapStyle::Auto,
        l::text_flow::TextWrapStyle::Balance,
    ] {
        let tree = fixture::<S>(direction, style);
        for inside in [
            l::inline_wrapping::WrapInside::Auto,
            l::inline_wrapping::WrapInside::Avoid,
        ] {
            let balanced = style == l::text_flow::TextWrapStyle::Balance
                && inside == l::inline_wrapping::WrapInside::Auto;
            let batch = compute(&tree, inside);
            for entries in [
                crate::layout::observation::phase(&batch, false),
                crate::layout::observation::phase(&batch, true),
            ] {
                let root = node(entries, 0);
                assert_eq!(root.size.width, S::from_f64(60.0));
                let receiving_y = node(entries, 6).location.y;
                assert!(receiving_y > S::ZERO);
                let expected_ltr_x = if balanced {
                    [0.0, 0.0, 20.0, 30.0]
                } else {
                    [0.0, 30.0, 50.0, 0.0]
                };
                // All supplied RTL levels are1. L2 reverses the actual slots;
                // Start alignment anchors each row at physical right60.
                let expected_rtl_x = if balanced {
                    [30.0, 40.0, 30.0, 20.0]
                } else {
                    [30.0, 10.0, 0.0, 50.0]
                };
                for (index, (id, width)) in [(2, 30.0), (3, 20.0), (5, 10.0), (6, 10.0)]
                    .into_iter()
                    .enumerate()
                {
                    let actual = node_of_kind(
                        entries,
                        id,
                        crate::layout::observation::expected_kind(
                            l::tree::LayoutTree::layout_input(&tree, id),
                            false,
                        ),
                    );
                    assert_eq!(
                        actual.source_index,
                        l::node::SourceIndex::new((id - 1) as usize)
                    );
                    assert_eq!(
                        actual.size,
                        l::geometry::Size::new(S::from_f64(width), S::from_f64(10.0))
                    );
                    if direction == l::geometry::Direction::Ltr {
                        assert_eq!(
                            actual.location.x,
                            S::from_f64(expected_ltr_x[index]),
                            "direction={direction:?} style={style:?} inside={inside:?} node={id} actual={actual:?}"
                        );
                    } else {
                        assert_eq!(
                            actual.location.x,
                            S::from_f64(expected_rtl_x[index]),
                            "direction={direction:?} style={style:?} inside={inside:?} node={id} actual={actual:?}"
                        );
                    }
                    let receiving = id == 6 || (balanced && id != 2);
                    assert_eq!(
                        actual.location.y,
                        if receiving { receiving_y } else { S::ZERO },
                        "direction={direction:?} style={style:?} inside={inside:?} node={id} actual={actual:?}"
                    );
                }
                // The selection oracle is independent of each numeric slot's
                // visual permutation: retain literal physical row hulls and
                // widths in both actual containing-axis profiles.
                let first: &[u32] = if balanced { &[2] } else { &[2, 3, 5] };
                let second: &[u32] = if balanced { &[3, 5, 6] } else { &[6] };
                for (ids, width) in [
                    (first, if balanced { 30.0 } else { 60.0 }),
                    (second, if balanced { 40.0 } else { 10.0 }),
                ] {
                    let actual = ids
                        .iter()
                        .map(|&id| {
                            node_of_kind(
                                entries,
                                id,
                                crate::layout::observation::expected_kind(
                                    l::tree::LayoutTree::layout_input(&tree, id),
                                    false,
                                ),
                            )
                        })
                        .collect::<Vec<_>>();
                    let left = actual
                        .iter()
                        .map(|item| item.location.x)
                        .reduce(S::min)
                        .unwrap();
                    let right = actual
                        .iter()
                        .map(|item| item.location.x + item.size.width)
                        .reduce(S::max)
                        .unwrap();
                    let used = actual
                        .iter()
                        .fold(S::ZERO, |sum, item| sum + item.size.width);
                    let start = if direction == l::geometry::Direction::Ltr {
                        0.0
                    } else {
                        60.0 - width
                    };
                    assert_eq!(used, S::from_f64(width));
                    assert_eq!(left, S::from_f64(start));
                    assert_eq!(right, S::from_f64(start + width));
                }
                let start = node_of_kind(
                    entries,
                    1,
                    crate::layout::observation::Kind::NonBox(
                        l::fragmentation::NonBoxKind::InlineBoundary,
                    ),
                );
                let end = node_of_kind(
                    entries,
                    4,
                    crate::layout::observation::Kind::NonBox(
                        l::fragmentation::NonBoxKind::InlineBoundary,
                    ),
                );
                for (id, output) in [(1, start), (4, end)] {
                    assert_eq!(
                        output.source_index,
                        l::node::SourceIndex::new((id - 1) as usize)
                    );
                    assert_eq!(output.size, l::geometry::Size::ZERO);
                }
                // A boundary's output is its actual baseline attachment, not
                // the top of an atomic box. Compare custody without guessing
                // the atomic's synthesized baseline or total block height.
                assert!(start.location.y > S::ZERO);
                if balanced {
                    assert!(end.location.y > start.location.y);
                    assert!(end.location.y >= receiving_y);
                } else {
                    assert_eq!(end.location.y, start.location.y);
                    assert!(end.location.y < receiving_y);
                }
                if direction == l::geometry::Direction::Ltr {
                    assert_eq!(
                        end.location.x,
                        S::from_f64(if balanced { 20.0 } else { 50.0 })
                    );
                } else {
                    // End is between B and C in the actual logical stream.
                    // Its zero-width L2 attachment is10 on ABC,40 on BCD.
                    assert_eq!(
                        end.location.x,
                        S::from_f64(if balanced { 40.0 } else { 10.0 })
                    );
                }
            }
        }
    }
}

#[test]
fn native_scope_priority_precedes_balanced_numeric_line_lengths() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        qualify::<f32>(direction);
        qualify::<f64>(direction);
    }
}

mod avoid_short_last_line;

mod pretty;
