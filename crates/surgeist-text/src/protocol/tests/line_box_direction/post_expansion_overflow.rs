//! Text4 7.1 applies resolved-start overflow alignment after real expansion.
use super::*;

#[test]
fn real_space_expansion_past_the_outer_width_overflows_only_the_resolved_end() {
    for body in BODIES {
        let p = capture("A A", body, WhiteSpace::Preserve, false);
        let paragraph_end = p
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        for (direction, start, a_positions) in [
            (BOXES[0], 0.0, [0.0, 40.0]),
            (BOXES[1], -15.0, [-15.0, 25.0]),
        ] {
            for alignment in [
                Alignment::Left,
                Alignment::Right,
                Alignment::Center,
                Alignment::Justify,
            ] {
                let description =
                    format!("body={body:?}, box={direction:?}, alignment={alignment:?}");
                let mut r = context(&p, direction, Some(35.0), 0.0, alignment);
                r.context.alignment_insets = LineAlignmentInsets::ZERO;
                r.context.spacing = SpacingPolicy::Spaces;
                let selected = p.select(r).unwrap();
                assert_eq!(
                    selected.request().termination,
                    PreparedTermination::Ordinary(paragraph_end.clone()),
                    "{description}"
                );
                assert_eq!(selected.request().end, paragraph_end.view().boundary());
                assert_eq!(selected.request().context.start, LineStart::Paragraph);
                assert_eq!(selected.request().context.end, LineEnd::Paragraph);
                let natural = selected
                    .finalize(crate::LineAllocations {
                        spacing: SpacingAllocations::None,
                        scopes: crate::ScopeAllocations::None,
                    })
                    .unwrap();
                // Real Ahem10 has A10 + SPACE10 + A10, which fits width35.
                assert_eq!(natural.feedback().advance, 30.0, "{description}");
                assert_eq!(natural.measurements().alignment().advance(), 30.0);
                let slots = selected.spacing_slots().collect::<Vec<_>>();
                assert_eq!(slots.len(), 1, "{description}");
                assert_eq!(
                    slots[0]
                        .pieces()
                        .map(|piece| piece.owner())
                        .collect::<Vec<_>>(),
                    [crate::SpacingSlotOwner::Source {
                        origin: p.origin_views().next().unwrap().handle(),
                        local: Range::new(1, 2),
                        aggregate: Range::new(1, 2),
                    }]
                );
                let expanded = selected
                    .finalize(crate::LineAllocations {
                        spacing: SpacingAllocations::Complete(vec![SpacingAllocation {
                            slot: slots[0].clone(),
                            amount: 20.0,
                        }]),
                        scopes: crate::ScopeAllocations::None,
                    })
                    .unwrap();

                // These physical positions distinguish overflow introduced by
                // actual expansion from overflow in the unexpanded selection.
                let expanded_glyphs = glyphs(&expanded);
                let actual_as = expanded_glyphs
                    .iter()
                    .filter(|glyph| glyph.0 == Range::new(0, 1) || glyph.0 == Range::new(2, 3))
                    .copied()
                    .collect::<Vec<_>>();
                assert_eq!(
                    actual_as,
                    [
                        (Range::new(0, 1), 35, 10.0, a_positions[0]),
                        (Range::new(2, 3), 35, 10.0, a_positions[1]),
                    ],
                    "{description}"
                );
                assert_eq!(actual_as[0].3, start);
                assert_eq!(actual_as[1].3 + f64::from(actual_as[1].2), start + 50.0);
                assert_eq!(expanded.feedback().advance, 50.0, "{description}");
                assert_eq!(expanded.measurements().alignment().advance(), 50.0);
                assert_eq!(expanded.feedback().line_height, 10.0);
                assert_eq!(expanded.feedback().baseline, 8.0);
                assert_eq!(expanded.selected_line(), selected);
                assert_eq!(expanded.prepared_text(), p.clone());
                let expanded_selection = expanded.selected_line();
                let actual = &expanded_selection.request().context;
                assert_eq!(actual.base_direction, body);
                assert_eq!(actual.line_box_direction, direction);
                assert_eq!(actual.alignment, alignment);
                assert_eq!(actual.available_inline, Some(35.0));
                assert_eq!(actual.indent, 0.0);
                assert_eq!(actual.alignment_insets, LineAlignmentInsets::ZERO);
                assert_eq!(actual.spacing, SpacingPolicy::Spaces);

                // Allocation changes spacing, not original glyph/source identity
                // or font advances. The helper authenticates actual Ahem bytes.
                assert_eq!(
                    expanded_glyphs
                        .iter()
                        .map(|g| (g.0, g.1, g.2))
                        .collect::<Vec<_>>(),
                    glyphs(&natural)
                        .iter()
                        .map(|g| (g.0, g.1, g.2))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    expanded_glyphs.iter().map(|g| g.2).collect::<Vec<_>>(),
                    [10.0; 3]
                );
                assert_eq!(
                    expanded
                        .mapping_views()
                        .map(|m| (m.source_range(), m.presentation_range()))
                        .collect::<Vec<_>>(),
                    natural
                        .mapping_views()
                        .map(|m| (m.source_range(), m.presentation_range()))
                        .collect::<Vec<_>>()
                );
                let original_levels = |line: &FinalizedLine| {
                    line.inner
                        .data
                        .clusters
                        .iter()
                        .map(|cluster| (cluster.source, cluster.level))
                        .collect::<Vec<_>>()
                };
                assert_eq!(original_levels(&expanded), original_levels(&natural));

                let layout = place(p.clone(), expanded, 0.0, 8.0).unwrap();
                assert_eq!(layout.source(), p.analyzed_source().source());
                assert_eq!(layout.source().text().code_units(), [65, 32, 65]);
                assert_eq!(layout.lines().len(), 1);
                assert_eq!(layout.lines()[0].range(), Range::new(0, 3));
                assert_eq!(layout.lines()[0].bounds(), Rect::new(0.0, 0.0, 35.0, 10.0));
                assert_eq!(layout.lines()[0].baseline(), 8.0);
                // Each actual A's source selection covers its ten-unit glyph
                // at the same expanded physical location, including overflow.
                for (range, x) in [
                    (Range::new(0, 1), a_positions[0]),
                    (Range::new(2, 3), a_positions[1]),
                ] {
                    let selection = layout
                        .selection(Selection::new(
                            Cursor::new(range.start, Affinity::After),
                            Cursor::new(range.end, Affinity::Before),
                        ))
                        .unwrap();
                    assert_eq!(selection.rects().len(), 1, "{description}");
                    assert_eq!(selection.rects()[0].line(), 0);
                    assert_eq!(
                        selection.rects()[0].rect(),
                        Rect::new(x, 0.0, 10.0, 10.0),
                        "{description}"
                    );
                }
                for (index, affinity, x) in [
                    (0, Affinity::After, start),
                    (1, Affinity::Before, start + 10.0),
                    (2, Affinity::After, start + 40.0),
                    (3, Affinity::Before, start + 50.0),
                ] {
                    let caret = layout.cursor(Cursor::new(index, affinity)).unwrap().rect();
                    assert_eq!(caret.origin.x, x, "{description}/{index}/{affinity:?}");
                    assert_eq!(caret.origin.y, 0.0);
                    assert_eq!(caret.size.height, 10.0);
                }
                // The previously finalized natural answer remains unchanged.
                assert_eq!(natural.feedback().advance, 30.0);
            }
        }
    }
}
