//! Actual source SHY eligibility consumed by native Layout, without emergency
//! wrapping or a fabricated control. Ahem's a/b/X are67/68/58 at10px each.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn style(word_break: text::WordBreak) -> text::Style {
    text::Style {
        white_space: text::WhiteSpace::Preserve,
        word_break,
        wrap: Wrap::Word,
        overflow_wrap: text::OverflowWrap::Normal,
        hyphens: text::Hyphens::Manual,
        hyphenate_character: text::HyphenateCharacter::String(text::Text::from("X")),
        hyphenate_limit_chars: text::HyphenateLimitChars {
            total: text::HyphenationMinimum::Count(0),
            before: text::HyphenationMinimum::Count(0),
            after: text::HyphenationMinimum::Count(0),
        },
        ..ahem_style(10.0, 10.0)
    }
}

fn paragraph(owner: text::WordBreak, following: text::WordBreak, rtl: bool) -> PreparedParagraph {
    let own = style(owner);
    let mut source = Source::identified("a\u{ad}b", text::Id::from_u64(65051), 7);
    source.span(text::Range::new(1, 2), own.clone());
    source.span(text::Range::new(2, 3), style(following));
    PreparedParagraph::new(
        vec![ParagraphPart::text(1, source, own.clone())],
        own,
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    )
}

fn inventory(
    paragraph: &PreparedParagraph,
    kind: Option<text::PreparedBreakKind>,
    manual: bool,
) -> text::PreparedBoundary {
    let p = &paragraph.prepared;
    let origin = paragraph.origin(1);
    assert_eq!(origin.view().source().id(), Some(text::Id::from_u64(65051)));
    assert_eq!(origin.view().source().revision(), 7);
    assert_eq!(origin.view().source().text().code_units(), [97, 0xad, 98]);
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        [97, 0xad, 98]
    );
    assert!(
        p.participants()
            .all(|row| !matches!(row, text::PreparedParticipant::Control(_)))
    );
    let cuts = p.hyphenation_cut_views().collect::<Vec<_>>();
    assert_eq!(cuts.len(), 1);
    assert_eq!(cuts[0].provenance(), text::HyphenationProvenance::Source);
    let cut = cuts[0].boundary();
    assert_eq!(cut.view().source_before().get(), 2);
    assert_eq!(cut.view().source_after().get(), 2);
    assert_eq!(cut.view().participant_offset(), 2);
    let offers = p
        .opportunity_views()
        .filter(|row| row.kind() != text::PreparedBreakKind::ParagraphEnd)
        .collect::<Vec<_>>();
    assert_eq!(
        offers.iter().map(|row| row.kind()).collect::<Vec<_>>(),
        kind.into_iter().collect::<Vec<_>>()
    );
    for offer in offers {
        assert_eq!(offer.boundary(), cut);
        let alternatives = offer.alternatives().collect::<Vec<_>>();
        assert_eq!(alternatives.len(), usize::from(manual));
        for alternative in alternatives {
            assert!(
                matches!(alternative.definition(), text::PreparedAlternativeDefinition::Source { origin: actual_origin, source } if actual_origin == origin && source == text::Range::new(1, 2))
            );
            assert_eq!(alternative.hyphenation_cut(), cuts[0].handle());
            assert_eq!(alternative.insertion_anchor().get(), 2);
            assert_eq!(alternative.text(), &text::Text::from("X"));
        }
    }
    cut
}

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    rtl: bool,
    width: AvailableOf<S>,
    limit: HyphenateLimitLines,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                hyphenate_limit_lines: limit,
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn assert_plain<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    cut: &text::PreparedBoundary,
    split: bool,
) {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), [97, 0xad, 98]);
    assert_eq!(placed.metrics().line_count(), if split { 2 } else { 1 });
    let mut lines = Vec::new();
    for entry in batch.unrounded_inline_fragments() {
        let line = entry.fragment().finalized_fragment().finalized_line();
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    assert_eq!(lines.len(), if split { 2 } else { 1 });
    let mut glyphs = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        // Placed line bounds describe the available allocation, so the
        // overflowing plain20px case still has a10px allocation. Inspect the
        // actual committed run advances for the raw contribution instead.
        let placed_metrics = placed.inline_metric_facts();
        let placed_raw: f64 = placed_metrics
            .runs()
            .iter()
            .filter(|run| run.line() == index)
            .map(|run| run.advance())
            .sum();
        assert_eq!(placed_raw, if split { 10.0 } else { 20.0 });
        assert_eq!(placed.lines()[index].baseline(), 8.0 + index as f64 * 10.0);
        assert_eq!(line.prepared_text(), paragraph.prepared);
        let selected = line.selected_line();
        let request = selected.request();
        assert!(
            request.alternative.is_none(),
            "a plain selection must not acquire the admitted manual sibling"
        );
        assert!(request.incoming.is_none());
        assert!(line.hyphenation_continuation().is_none());
        assert_eq!(line.feedback().advance, if split { 10.0 } else { 20.0 });
        assert_eq!(
            line.measurements().fit().advance(),
            if split { 10.0 } else { 20.0 }
        );
        if split && index == 0 {
            let text::PreparedTermination::Ordinary(offer) = &request.termination else {
                panic!("the actual ordinary after-SHY offer must be selected")
            };
            assert_eq!(offer.view().kind(), text::PreparedBreakKind::Allowed);
            assert_eq!(offer.view().boundary(), *cut);
            assert_eq!(request.end, *cut);
            assert_eq!(request.context.end, text::LineEnd::Soft);
        } else {
            assert_eq!(request.context.end, text::LineEnd::Paragraph);
        }
        if split && index == 1 {
            assert_eq!(request.start, *cut);
        }
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.font_size(), 10.0);
            for glyph in run.glyphs() {
                let outline = instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap()
                    .bounds()
                    .unwrap();
                assert!(outline.size.width > 0.0 && outline.size.height > 0.0);
                glyphs.push((index, glyph.range(), glyph.id(), glyph.advance()));
            }
        }
    }
    glyphs.sort_by_key(|row| (row.1.start, row.1.end));
    assert_eq!(
        glyphs,
        [
            (0, text::Range::new(0, 1), 67, 10.0),
            (usize::from(split), text::Range::new(2, 3), 68, 10.0)
        ]
    );
    let mut placed_glyphs = placed
        .run_views()
        .flat_map(|run| {
            let line = run.line();
            run.glyphs()
                .map(move |glyph| (line, glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    placed_glyphs.sort_by_key(|row| (row.0, row.1));
    let mut expected = glyphs
        .iter()
        .map(|row| (row.0, row.2, row.3))
        .collect::<Vec<_>>();
    expected.sort_by_key(|row| (row.0, row.1));
    assert_eq!(placed_glyphs, expected);
}

#[test]
fn source_shy_mixed_break_all_owners_drive_plain_native_lines_both_directions_and_scalars() {
    fn lane<S: LayoutScalar>() {
        use text::WordBreak::{BreakAll, Normal};
        for rtl in [false, true] {
            for (owner, following, manual, split) in [
                (BreakAll, BreakAll, false, true),
                (Normal, BreakAll, true, true),
                (BreakAll, Normal, false, false),
            ] {
                let p = paragraph(owner, following, rtl);
                let cut = inventory(
                    &p,
                    split.then_some(text::PreparedBreakKind::Allowed),
                    manual,
                );
                // A zero successive-line limit suppresses Auto only. Manual X
                // remains admitted in Normal/BA, yet its20px ending cannot fit
                // this10px line; its real None sibling remains selectable.
                for limit in [HyphenateLimitLines::NoLimit, HyphenateLimitLines::Count(0)] {
                    let batch =
                        compute::<S>(&p, rtl, AvailableOf::definite(S::from_f64(10.0)), limit);
                    assert_plain(&batch, &p, &cut, split);
                    for (available, expected) in [
                        (AvailableOf::MIN_CONTENT, if split { 10.0 } else { 20.0 }),
                        (AvailableOf::MAX_CONTENT, 20.0),
                    ] {
                        let intrinsic = compute::<S>(&p, rtl, available, limit);
                        let root = intrinsic
                            .unrounded_entries()
                            .iter()
                            .find(|entry| entry.node() == 0)
                            .unwrap()
                            .output();
                        assert_eq!(
                            root.size.width,
                            S::from_f64(expected),
                            "actual intrinsic consumer owner={owner:?} following={following:?} available={available:?}"
                        );
                    }
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn normal_shy_keeps_manual_only_inventory_without_promoting_a_plain_cut() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            let p = paragraph(text::WordBreak::Normal, text::WordBreak::Normal, rtl);
            let cut = inventory(&p, Some(text::PreparedBreakKind::Discretionary), true);
            // Both source letters fit20px. Count0 does not revoke Manual;
            // Layout legitimately chooses the later whole-word plain ending.
            let batch = compute::<S>(
                &p,
                rtl,
                AvailableOf::definite(S::from_f64(20.0)),
                HyphenateLimitLines::Count(0),
            );
            assert_plain(&batch, &p, &cut, false);
            for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
                let intrinsic = compute::<S>(&p, rtl, available, HyphenateLimitLines::Count(0));
                let root = intrinsic
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output();
                assert_eq!(root.size.width, S::from_f64(20.0));
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}
