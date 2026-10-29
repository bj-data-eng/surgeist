//! Native scope avoidance precedes Balance's preference among legal numeric cuts.
mod stable;

use super::*;

fn fixture<S: l::LayoutScalar>(direction: l::Direction, style: l::TextWrapStyle) -> Tree<S> {
    let atomic = |width| {
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::InlineBlock,
            direction,
            vertical_align: l::VerticalAlignOf::Baseline,
            atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
                t::BidiLevel::try_new(if direction == l::Direction::Ltr { 0 } else { 1 }).unwrap(),
                l::InlineBreakOpportunity::Allowed,
            )),
            size: l::Size::new(
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(width)).unwrap()),
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(10.0)).unwrap()),
            ),
            ..l::NodeInputOf::default()
        })
    };
    let boundary = |kind| {
        l::LayoutInputOf::inline_boundary(
            l::InlineBoundaryInputOf::new(
                kind,
                l::InlineMetricsOf::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                l::InlineBoundaryEdgeOf::ZERO,
            )
            .with_direction(direction),
        )
    };
    Tree {
        inputs: vec![
            l::LayoutInputOf::box_input(l::NodeInputOf {
                display: l::Display::Block,
                direction,
                text_align: l::TextAlign::Start,
                text_align_last: l::TextAlignLast::Start,
                text_wrap_style: style,
                size: l::Size::new(
                    l::PreferredSizeOf::value(
                        l::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                    ),
                    l::PreferredSizeOf::AUTO,
                ),
                ..l::NodeInputOf::default()
            }),
            boundary(l::InlineBoundaryKind::Start),
            atomic(30.0),
            atomic(20.0),
            boundary(l::InlineBoundaryKind::End),
            atomic(10.0),
            atomic(10.0),
        ],
        non_box: l::NodeInputOf::non_box(),
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

fn compute<S: l::LayoutScalar>(
    tree: &Tree<S>,
    inside: l::WrapInside,
) -> l::CompletedLayoutBatchOf<u32, S> {
    let flow = l::LayoutFlowInput::try_new(
        tree,
        0,
        Vec::new(),
        l::WrappingInput {
            scopes: vec![l::WrappingScopeInput::NativePair {
                start: 1,
                end: 4,
                wrap_inside: inside,
                edges: l::WrapEdges::AUTO,
                // The actual pair has these same Baseline metrics. The witness
                // identifies that contribution; it must not duplicate it.
                metric_contribution: Some(l::InlineBoxMetricContribution::Strut(t::LineBoxStrut {
                    metrics: t::LineStrut {
                        baseline: 8.0,
                        line_extent: 10.0,
                    },
                    alignment: t::LineBoxAlignment::Baseline,
                })),
            }],
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request = l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(S::from_f64(60.0)),
        l::AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = l::compute_layout_with_flow(&flow, 0, request).unwrap();
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    batch
}

fn node<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    id: u32,
) -> l::NodeOutputOf<S> {
    let mut matching = entries.iter().filter(|entry| entry.node() == id);
    let output = matching.next().unwrap().output();
    assert!(matching.next().is_none());
    output
}

fn qualify<S: l::LayoutScalar>(direction: l::Direction) {
    // Real source order is Start, A30, B20, End, C10, D10. Every
    // atomic supplies its own following Allowed cut; neither endpoint does.
    // Auto takes ABC / D = 60/10. Balance can improve to A / BCD = 30/40:
    // squared deviations from the two-line mean35 are 50 rather than1250.
    // Cuts after A and B are inside the actual pair. The after-C cut is
    // outside it, so its empty violation set strictly dominates either inside
    // cut when Avoid is requested, before Balance compares line lengths.
    for style in [l::TextWrapStyle::Auto, l::TextWrapStyle::Balance] {
        let tree = fixture::<S>(direction, style);
        for inside in [l::WrapInside::Auto, l::WrapInside::Avoid] {
            let balanced = style == l::TextWrapStyle::Balance && inside == l::WrapInside::Auto;
            let batch = compute(&tree, inside);
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
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
                    let actual = node(entries, id);
                    assert_eq!(actual.source_index, l::SourceIndex::new((id - 1) as usize));
                    assert_eq!(
                        actual.size,
                        l::Size::new(S::from_f64(width), S::from_f64(10.0))
                    );
                    if direction == l::Direction::Ltr {
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
                    let actual = ids.iter().map(|&id| node(entries, id)).collect::<Vec<_>>();
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
                    let start = if direction == l::Direction::Ltr {
                        0.0
                    } else {
                        60.0 - width
                    };
                    assert_eq!(used, S::from_f64(width));
                    assert_eq!(left, S::from_f64(start));
                    assert_eq!(right, S::from_f64(start + width));
                }
                let start = node(entries, 1);
                let end = node(entries, 4);
                for (id, output) in [(1, start), (4, end)] {
                    assert_eq!(output.source_index, l::SourceIndex::new((id - 1) as usize));
                    assert_eq!(output.size, l::Size::ZERO);
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
                if direction == l::Direction::Ltr {
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
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        qualify::<f32>(direction);
        qualify::<f64>(direction);
    }
}

mod avoid_short_last_line;

mod pretty;
