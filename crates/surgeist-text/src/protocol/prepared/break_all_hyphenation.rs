//! Actual source SHY permission is independent from the indication's owner.
use super::*;
use crate::{
    HyphenateCharacter, HyphenateLimitChars, HyphenationMinimum::Count, Hyphens, WordBreak, Wrap,
};

fn style(mode: WordBreak, indication: &str) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        white_space: WhiteSpace::Preserve,
        wrap: Wrap::Word,
        word_break: mode,
        hyphens: Hyphens::Manual,
        hyphenate_character: HyphenateCharacter::String(Text::from(indication)),
        hyphenate_limit_chars: HyphenateLimitChars {
            total: Count(0),
            before: Count(0),
            after: Count(0),
        },
        ..Style::default()
    }
}

fn prepare(input: ParagraphInput, ancestor: Wrap) -> PreparedText {
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input).unwrap(),
        ancestor,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::mark_ahem())
        .unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn offers(p: &PreparedText) -> Vec<(usize, usize, PreparedBreakKind, usize)> {
    p.opportunity_views()
        .filter(|offer| offer.kind() != PreparedBreakKind::ParagraphEnd)
        .map(|offer| {
            (
                offer.boundary().view().participant_offset(),
                offer.boundary().view().source_before().get(),
                offer.kind(),
                offer.alternatives().count(),
            )
        })
        .collect()
}

fn selection(
    p: &PreparedText,
    offer: PreparedOpportunity,
    direction: ParagraphBaseDirection,
) -> LineSelectionRequest {
    let end = offer.view().boundary();
    let mut context = context(LineEnd::Soft);
    context.base_direction = direction;
    context.line_box_direction = match direction {
        crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
        crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
    };
    context.spacing = SpacingPolicy::None;
    LineSelectionRequest {
        start: p.boundary_views().next().unwrap().handle(),
        participants: p
            .participants()
            .take(end.view().participant_offset())
            .collect(),
        end,
        termination: PreparedTermination::Ordinary(offer),
        alternative: None,
        incoming: None,
        context,
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
    }
}

#[test]
fn shy_local_wrap_cannot_lend_an_ordinary_cut_its_missing_ancestor_permission() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        for owner in [WordBreak::Normal, WordBreak::BreakAll] {
            let mut source = Source::new("A\u{ad}B");
            source.span(Range::new(2, 3), style(WordBreak::BreakAll, "X"));
            let mut input = paragraph(source.clone());
            input.base_style = style(owner, "X");
            input.base_direction = direction;
            let p = prepare(input, Wrap::None);
            let cut = p
                .hyphenation_cut_views()
                .find(|cut| cut.provenance() == HyphenationProvenance::Source)
                .unwrap();
            assert_eq!(cut.style().wrap, Wrap::Word);
            assert_eq!(cut.boundary().view().source_before().get(), 2);
            assert_eq!(
                offers(&p),
                if owner == WordBreak::Normal {
                    vec![(2, 2, PreparedBreakKind::Discretionary, 1)]
                } else {
                    vec![]
                }
            );
            assert_eq!(p.analyzed_source().source(), &source);
            if owner == WordBreak::Normal {
                let offer = cut.opportunity().unwrap();
                let mut request = selection(&p, offer.clone(), direction);
                assert!(matches!(
                    p.select(request.clone()).unwrap_err().detail(),
                    Some(crate::ErrorDetail::InvalidTextProtocol {
                        reason: ProtocolError::MissingAlternative { .. }
                    })
                ));
                request.alternative = Some(offer.view().alternatives().next().unwrap().handle());
                let line = p
                    .select(request)
                    .unwrap()
                    .finalize(SpacingAllocations::None)
                    .unwrap();
                assert_eq!(line.feedback().advance, 20.0);
            }
        }
    }
}

#[test]
fn ordinary_shy_cuts_keep_real_ba_punctuation_joiner_and_typographic_context() {
    // LB21 forbids before a second BA; LB13 forbids before EX. LB11/LB8a
    // preserve WJ/ZWJ, while the following base+mark remains one genuine unit.
    for (value, expected) in [
        ("A\u{ad}\u{ad}B", vec![3]),
        ("A\u{ad}!", vec![]),
        ("A\u{ad}B\u{301}", vec![2]),
        ("A\u{ad}\u{2060}B", vec![]),
        ("A\u{ad}\u{200d}B", vec![]),
    ] {
        let mut input = paragraph(Source::new(value));
        input.base_style = style(WordBreak::BreakAll, "X");
        let p = prepare(input, Wrap::Word);
        assert_eq!(
            offers(&p)
                .into_iter()
                .map(|(_, source, kind, alternatives)| {
                    assert_eq!((kind, alternatives), (PreparedBreakKind::Allowed, 0));
                    source
                })
                .collect::<Vec<_>>(),
            expected,
            "{value:?}"
        );
        assert_eq!(p.alternative_views().count(), 0);
    }
}

#[test]
fn later_source_free_anchors_cannot_copy_the_actual_shy_cut() {
    let mut input = paragraph(Source::new("A\u{ad}B"));
    input.base_style = style(WordBreak::BreakAll, "X");
    input.controls = [
        ParagraphControlKind::InlineBoundaryEnd,
        ParagraphControlKind::RightToLeftEmbedding,
        ParagraphControlKind::PopDirectionalFormat,
    ]
    .into_iter()
    .map(|kind| ParagraphControl { at: 2, kind })
    .collect();
    input.anchor_order = (0..3)
        .map(|control| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(control),
            affinity: Affinity::After,
        })
        .collect();
    let p = prepare(input, Wrap::Word);
    let equal_offsets: Vec<_> = p
        .boundary_views()
        .filter(|cut| cut.source_before().get() == 2)
        .map(|cut| cut.participant_offset())
        .collect();
    assert_eq!(equal_offsets, [2, 3, 4, 5]);
    assert_eq!(offers(&p), [(2, 2, PreparedBreakKind::Allowed, 0)]);
}

#[test]
fn normal_empty_manual_indication_is_an_admitted_choice_and_partition() {
    let mut input = paragraph(Source::new("A\u{ad}B"));
    input.base_style = style(WordBreak::Normal, "");
    let p = prepare(input, Wrap::Word);
    assert_eq!(offers(&p), [(2, 2, PreparedBreakKind::Discretionary, 1)]);
    let portions = p.word_views().next().unwrap().portions().unwrap();
    assert_eq!(portions.len(), 2);
    assert!(
        portions
            .iter()
            .all(PreparedWordPortion::has_manual_partition)
    );
    let offer = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Discretionary)
        .unwrap()
        .handle();
    let alternative = offer.view().alternatives().next().unwrap().handle();
    let mut request = selection(&p, offer, ParagraphBaseDirection::LeftToRight);
    request.alternative = Some(alternative.clone());
    let line = p
        .select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(
        line.selected_line().request().alternative,
        Some(alternative)
    );
    assert!(line.hyphenation_continuation().is_none());
}

#[test]
fn standalone_break_all_shy_keeps_ordinary_flow_without_an_indication() {
    let source = Source::new("A\u{ad}B");
    let layout = system()
        .layout(
            source.clone(),
            style(WordBreak::BreakAll, "X"),
            crate::Options {
                width: Some(10.0),
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    assert_eq!(layout.source(), &source);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let glyphs: Vec<_> = layout
        .glyph_runs()
        .iter()
        .flat_map(|run| {
            run.glyphs()
                .iter()
                .map(|glyph| (glyph.range(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(glyphs, [(Range::new(0, 1), 10.0), (Range::new(2, 3), 10.0)]);
}

#[test]
fn atomic_ordinary_permission_unions_with_manual_without_vetoing_it_when_false() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        for ancestor in [Wrap::Word, Wrap::None] {
            let mut source = Source::new("A\u{ad}B");
            source.span(Range::new(2, 3), style(WordBreak::BreakAll, "X"));
            source.inline_box(InlineBox::new(
                Id::from_u64(65011),
                InlineBoxKind::InFlow,
                2,
                Size::new(10.0, 10.0),
            ));
            let mut input = paragraph(source.clone());
            input.base_style = style(WordBreak::Normal, "X");
            input.base_direction = direction;
            input.anchor_order = vec![ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(0),
                affinity: Affinity::After,
            }];
            input.boundary_policies = [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: ancestor,
            })
            .collect();
            let p = prepare(input, ancestor);
            assert_eq!(
                offers(&p),
                if ancestor == Wrap::Word {
                    vec![
                        (2, 2, PreparedBreakKind::Allowed, 1),
                        (3, 2, PreparedBreakKind::Allowed, 0),
                    ]
                } else {
                    vec![(2, 2, PreparedBreakKind::Discretionary, 1)]
                }
            );
            let offer = p
                .opportunity_views()
                .find(|offer| offer.boundary().view().participant_offset() == 2)
                .unwrap()
                .handle();
            let request = selection(&p, offer.clone(), direction);
            if ancestor == Wrap::Word {
                let ordinary = p
                    .select(request.clone())
                    .unwrap()
                    .finalize(SpacingAllocations::None)
                    .unwrap();
                assert_eq!(ordinary.feedback().advance, 10.0);
                assert!(ordinary.selected_line().request().alternative.is_none());
            } else {
                assert!(matches!(
                    p.select(request.clone()).unwrap_err().detail(),
                    Some(crate::ErrorDetail::InvalidTextProtocol {
                        reason: ProtocolError::MissingAlternative { .. }
                    })
                ));
            }
            let mut manual = request;
            manual.alternative = Some(offer.view().alternatives().next().unwrap().handle());
            let line = p
                .select(manual)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap();
            assert_eq!(line.feedback().advance, 20.0);
            assert!(line.selected_line().request().alternative.is_some());
            assert_eq!(p.analyzed_source().source(), &source);
        }
    }
}

#[test]
fn out_of_flow_occurrence_does_not_hide_the_actual_following_text_owner() {
    for following in [WordBreak::Normal, WordBreak::BreakAll] {
        let mut source = Source::new("A\u{ad}B");
        source.span(Range::new(2, 3), style(following, "X"));
        source.inline_box(InlineBox::new(
            Id::from_u64(65012),
            InlineBoxKind::OutOfFlow,
            2,
            Size::new(3.0, 4.0),
        ));
        let mut input = paragraph(source.clone());
        input.base_style = style(WordBreak::BreakAll, "X");
        input.anchor_order = vec![ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(0),
            affinity: Affinity::After,
        }];
        let p = prepare(input, Wrap::Word);
        assert_eq!(
            offers(&p),
            if following == WordBreak::BreakAll {
                vec![(2, 2, PreparedBreakKind::Allowed, 0)]
            } else {
                vec![]
            }
        );
        assert_eq!(p.alternative_views().count(), 0);
        assert_eq!(p.analyzed_source().source(), &source);
    }
}

#[test]
fn standalone_atomic_fallback_does_not_lend_the_source_shy_an_oof_permission() {
    for following in [WordBreak::Normal, WordBreak::BreakAll] {
        let mut source = Source::new("A\u{ad}B");
        source.span(Range::new(2, 3), style(following, "X"));
        source.inline_box(InlineBox::new(
            Id::from_u64(65013),
            InlineBoxKind::OutOfFlow,
            2,
            Size::new(3.0, 4.0),
        ));
        let analyzed =
            AnalyzedSource::new(source.clone(), style(WordBreak::BreakAll, "X")).unwrap();
        let p = system()
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        // Observe the actual source SHY cut, not a later raw-profile atomic
        // side with the same numeric offset. Only the following text can lend
        // this source-owned cut its independent ordinary BreakAll permission.
        let at_source_shy: Vec<_> = offers(&p)
            .into_iter()
            .filter(|(participant, _, _, _)| *participant == 2)
            .collect();
        assert_eq!(
            at_source_shy,
            if following == WordBreak::BreakAll {
                vec![(2, 2, PreparedBreakKind::Allowed, 0)]
            } else {
                vec![]
            }
        );
        assert_eq!(p.alternative_views().count(), 0);
        assert_eq!(p.analyzed_source().source(), &source);
    }
}
