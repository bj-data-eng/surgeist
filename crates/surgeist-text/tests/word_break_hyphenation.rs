#![forbid(unsafe_code)]

//! Existing-callable word-break/hyphenation custody with a real registered font.
//! Pinned CSS Text4 forbids hyphenation under BreakAll. The retained en-US
//! source exception spells as-so-ciate, independently supplying cuts2/4.

use skrifa::MetadataProvider;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const WORD: &str = "associate";
const LETTERS: [u32; 9] = [67, 85, 85, 81, 69, 75, 67, 86, 71];

fn width(letters: usize) -> f64 {
    f64::from(u32::try_from(letters).unwrap()) * 10.0
}

fn style(word_break: t::WordBreak) -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        word_break,
        hyphens: t::Hyphens::Auto,
        locale: Some("en-US".into()),
        hyphenate_character: t::HyphenateCharacter::String(t::Text::from("X")),
        ..t::Style::default()
    }
}

fn paragraph(source: t::Source, base: t::Style) -> t::ParagraphInput {
    let end = source.text().len();
    t::ParagraphInput {
        source: source.clone(),
        base_style: base,
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: t::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: t::ParagraphStrut::DefaultFont,
        terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::ParagraphOrigin {
            text_node: None,
            source,
            local: t::Range::new(0, end),
            aggregate: t::Range::new(0, end),
            kind: t::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn prepare_input(input: t::ParagraphInput) -> t::PreparedText {
    let wrap = input.base_style.wrap;
    let analyzed =
        t::AnalyzedSource::from_paragraph(input, t::AnalysisTypography::HorizontalComposition)
            .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap()
}

fn prepare(word_break: t::WordBreak) -> (t::PreparedText, t::Source) {
    let source = t::Source::identified(t::Text::from(WORD), t::Id::from_u64(65001), 7);
    (
        prepare_input(paragraph(source.clone(), style(word_break))),
        source,
    )
}

fn glyphs(line: &t::FinalizedLine) -> Vec<(t::Range, u32, f32)> {
    let mut output = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 10.0);
        assert_eq!(
            instance
                .metrics(run.face().catalogue_identity())
                .unwrap()
                .units_per_em(),
            1000
        );
        for glyph in run.glyphs() {
            let ink = instance
                .glyph_outline(glyph.id(), run.face().catalogue_identity())
                .unwrap()
                .bounds()
                .unwrap();
            assert!(ink.size.width > 0.0 && ink.size.height > 0.0);
            output.push((glyph.range(), glyph.id(), glyph.advance()));
        }
    }
    output.sort_by_key(|row| (row.0.start, row.0.end));
    output
}

fn finish(
    p: &t::PreparedText,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    alternative: Option<t::PreparedAlternative>,
    available: f64,
) -> t::FinalizedLine {
    let boundary = end.view().boundary();
    let from = start.view().participant_offset();
    let to = boundary.view().participant_offset();
    let participants = p
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::PreparedParticipant::Control(occurrence) => Some(t::ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
                boundary: None,
            }),
            _ => None,
        })
        .collect();
    p.select(t::LineSelectionRequest {
        inline_boxes: None,
        segment: p.source_segments().next().unwrap(),
        start,
        end: boundary,
        participants,
        termination: t::PreparedTermination::Ordinary(end.clone()),
        alternative,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
        context: t::LineContext {
            alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
            formatted_line: surgeist_text::FormattedLineContext::default(),
            base_direction: if p.analyzed_source().direction() == t::Direction::RightToLeft {
                t::ParagraphBaseDirection::RightToLeft
            } else {
                t::ParagraphBaseDirection::LeftToRight
            },
            line_box_direction: if p.analyzed_source().direction() == t::Direction::RightToLeft {
                surgeist_text::LineBoxDirection::RightToLeft
            } else {
                surgeist_text::LineBoxDirection::LeftToRight
            },
            orientation: t::LineOrientation::Horizontal,
            start: if from == 0 {
                t::LineStart::Paragraph
            } else {
                t::LineStart::Continuation
            },
            end: match end.view().kind() {
                t::PreparedBreakKind::Forced => t::LineEnd::Forced,
                t::PreparedBreakKind::ParagraphEnd => t::LineEnd::Paragraph,
                _ => t::LineEnd::Soft,
            },
            available_inline: Some(available),
            indent: 0.0,
            alignment: t::Alignment::Start,
            tabs: t::TabContext::NoTabs,
            spacing: t::SpacingPolicy::None,
        },
    })
    .unwrap()
    .finalize(surgeist_text::LineAllocations {
        spacing: t::SpacingAllocations::None,
        scopes: surgeist_text::ScopeAllocations::None,
    })
    .unwrap()
}

fn start(p: &t::PreparedText) -> t::PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}

fn end(p: &t::PreparedText) -> t::PreparedOpportunity {
    p.opportunity_views()
        .find(|offer| offer.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}

fn automatic(p: &t::PreparedText) -> Vec<(usize, t::PreparedOpportunity, t::PreparedAlternative)> {
    p.opportunity_views()
        .flat_map(|offer| {
            let at = offer.boundary().view().source_before().get();
            offer
                .alternatives()
                .filter(|alternative| {
                    matches!(
                        alternative.definition(),
                        t::PreparedAlternativeDefinition::Automatic { .. }
                    )
                })
                .map(|alternative| (at, offer.handle(), alternative.handle()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn expected_letters(end: usize) -> Vec<(t::Range, u32, f32)> {
    LETTERS[..end]
        .iter()
        .enumerate()
        .map(|(index, &glyph)| (t::Range::new(index, index + 1), glyph, 10.0))
        .collect()
}

fn copy_whole(p: &t::PreparedText, line: t::FinalizedLine, original: &t::Source) {
    let layout = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor: t::LineAnchor {
                inline_origin: 0.0,
                baseline: 8.0,
            },
            orientation: t::LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), original);
    let before = layout
        .positions()
        .find(|position| position.source_cursor() == t::Cursor::new(0, t::Affinity::After))
        .unwrap();
    let after = layout
        .positions()
        .find(|position| {
            position.source_cursor() == t::Cursor::new(original.text().len(), t::Affinity::Before)
        })
        .unwrap();
    let selected = layout.position_selection(&before, &after).unwrap();
    assert_eq!(selected.source_text(), original.text());
    assert_eq!(selected.plaintext(), original.text());
}

fn qualify(p: &t::PreparedText, original: &t::Source) {
    let font = skrifa::FontRef::new(AHEM).unwrap();
    for (ch, glyph) in WORD.chars().zip(LETTERS) {
        assert_eq!(font.charmap().map(ch).unwrap().to_u32(), glyph);
    }
    assert_eq!(font.charmap().map('X').unwrap().to_u32(), 58);
    let analyzed = p.analyzed_source();
    assert_eq!(analyzed.source(), original);
    assert_eq!(
        analyzed.processed_source().processed_text(),
        &t::Text::from(WORD)
    );
    assert_eq!(original.id(), Some(t::Id::from_u64(65001)));
    assert_eq!(original.revision(), 7);
    let words = p.word_views().collect::<Vec<_>>();
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].query(), Some(WORD));
    assert_eq!(words[0].resource(), t::HyphenationResource::EnUs2005);
    assert_eq!(
        words[0]
            .source_pieces()
            .flat_map(|piece| piece.source().start..piece.source().end)
            .collect::<Vec<_>>(),
        (0..9).collect::<Vec<_>>()
    );
    let full = finish(p, start(p), end(p), None, 90.0);
    assert_eq!(full.feedback().advance, 90.0);
    assert_eq!(glyphs(&full), expected_letters(9));
    assert!(full.selected_line().request().alternative.is_none());
    copy_whole(p, full, original);
    assert_eq!(p.analyzed_source().source(), original);
}

#[test]
fn real_ahem_and_complete_en_us_word_are_qualified_under_each_uniform_word_break() {
    for word_break in [
        t::WordBreak::Normal,
        t::WordBreak::KeepAll,
        t::WordBreak::BreakAll,
    ] {
        let (p, original) = prepare(word_break);
        assert_eq!(p.analyzed_source().style().word_break, word_break);
        qualify(&p, &original);
    }
}

#[test]
fn normal_and_keep_all_retain_source_exception_auto_choices_and_real_selected_indication() {
    for word_break in [t::WordBreak::Normal, t::WordBreak::KeepAll] {
        let (p, original) = prepare(word_break);
        qualify(&p, &original);
        assert_eq!(
            p.word_views().next().unwrap().provenance(),
            Some(t::HyphenationProvenance::SourceException { entry: WORD })
        );
        let choices = automatic(&p);
        assert_eq!(
            choices.iter().map(|choice| choice.0).collect::<Vec<_>>(),
            [2, 4]
        );
        for (at, offer, alternative) in choices {
            assert_eq!(offer.view().kind(), t::PreparedBreakKind::Discretionary);
            assert_eq!(alternative.view().text(), &t::Text::from("X"));
            assert_eq!(alternative.view().source_range(), None);
            assert_eq!(alternative.view().insertion_anchor().get(), at);
            let first = finish(&p, start(&p), offer, Some(alternative), width(at + 1));
            assert_eq!(first.feedback().advance, width(at + 1));
            let mut expected = expected_letters(at);
            expected.push((t::Range::new(at, at), 58, 10.0));
            assert_eq!(glyphs(&first), expected);
            assert_eq!(p.analyzed_source().source(), &original);
        }
    }
}

#[test]
fn break_all_keeps_each_real_ordinary_letter_cut_and_selects_without_indication() {
    let (p, original) = prepare(t::WordBreak::BreakAll);
    qualify(&p, &original);
    let ordinary = p
        .opportunity_views()
        .filter(|offer| offer.kind() == t::PreparedBreakKind::Allowed)
        .map(|offer| offer.handle())
        .collect::<Vec<_>>();
    assert_eq!(
        ordinary
            .iter()
            .map(|offer| offer.view().boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        (1..9).collect::<Vec<_>>()
    );
    for offer in ordinary {
        let at = offer.view().boundary().view().source_before().get();
        let line = finish(&p, start(&p), offer, None, width(at));
        assert_eq!(line.feedback().advance, width(at));
        assert!(line.selected_line().request().alternative.is_none());
        assert_eq!(glyphs(&line), expected_letters(at));
        assert_eq!(p.analyzed_source().source(), &original);
    }
}

#[test]
fn break_all_publishes_no_eligible_automatic_alternative_at_any_letter_cut() {
    let (p, original) = prepare(t::WordBreak::BreakAll);
    qualify(&p, &original);
    // Truthful word/resource/cut diagnostics may remain. The violated contract
    // is publishing a selectable automatic alternative, not retaining CutData.
    let choices = automatic(&p);
    let diagnostic = choices
        .iter()
        .map(|(at, offer, alternative)| {
            let line = finish(
                &p,
                start(&p),
                offer.clone(),
                Some(alternative.clone()),
                width(at + 1),
            );
            (
                *at,
                line.feedback().advance,
                glyphs(&line),
                line.selected_line().request().alternative.is_some(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(p.analyzed_source().source(), &original);
    assert!(
        choices.is_empty(),
        "CSS Text4 BreakAll forbids automatic hyphenation; actual published and selected (cut, advance, glyphs, used_alternative)={diagnostic:?}"
    );
}

// The adopted mixed-owner determination assigns automatic eligibility to the
// actual logical-preceding text, independently of following-box ordinary cuts.
fn mixed_automatic(
    preceding: t::WordBreak,
    following: t::WordBreak,
) -> (t::PreparedText, t::Source) {
    let mut source = t::Source::identified(t::Text::from(WORD), t::Id::from_u64(65001), 7);
    source.span(t::Range::new(1, 2), style(preceding));
    source.span(t::Range::new(2, 3), style(following));
    (
        prepare_input(paragraph(source.clone(), style(t::WordBreak::Normal))),
        source,
    )
}

#[test]
fn automatic_break_all_preceding_owner_suppresses_only_its_actual_dictionary_cut() {
    let (p, source) = mixed_automatic(t::WordBreak::BreakAll, t::WordBreak::Normal);
    qualify(&p, &source);
    assert_eq!(
        automatic(&p).iter().map(|row| row.0).collect::<Vec<_>>(),
        [4]
    );
}

#[test]
fn automatic_normal_preceding_owner_survives_following_break_all_ordinary_authority() {
    let (p, source) = mixed_automatic(t::WordBreak::Normal, t::WordBreak::BreakAll);
    qualify(&p, &source);
    let choices = automatic(&p);
    assert_eq!(choices.iter().map(|row| row.0).collect::<Vec<_>>(), [2, 4]);
    let (_, offer, alternative) = &choices[0];
    assert_eq!(offer.view().kind(), t::PreparedBreakKind::Allowed);
    let ordinary = finish(&p, start(&p), offer.clone(), None, 20.0);
    assert_eq!(ordinary.feedback().advance, 20.0);
    assert_eq!(glyphs(&ordinary), expected_letters(2));
    let hyphenated = finish(
        &p,
        start(&p),
        offer.clone(),
        Some(alternative.clone()),
        30.0,
    );
    let mut expected = expected_letters(2);
    expected.push((t::Range::new(2, 2), 58, 10.0));
    assert_eq!(hyphenated.feedback().advance, 30.0);
    assert_eq!(glyphs(&hyphenated), expected);
}

fn manual_style(word_break: t::WordBreak) -> t::Style {
    t::Style {
        hyphens: t::Hyphens::Manual,
        hyphenate_limit_chars: t::HyphenateLimitChars {
            total: t::HyphenationMinimum::Count(0),
            before: t::HyphenationMinimum::Count(0),
            after: t::HyphenationMinimum::Count(0),
        },
        ..style(word_break)
    }
}

fn shy_input(
    owner: t::Style,
    following: t::WordBreak,
    direction: t::ParagraphBaseDirection,
) -> (t::PreparedText, t::Source) {
    let mut source = t::Source::identified(t::Text::from("a\u{ad}b"), t::Id::from_u64(65002), 11);
    source.span(t::Range::new(1, 2), owner.clone());
    let mut next = owner.clone();
    next.word_break = following;
    source.span(t::Range::new(2, 3), next);
    let mut input = paragraph(source.clone(), owner);
    input.base_direction = direction;
    (prepare_input(input), source)
}

fn directions() -> [t::ParagraphBaseDirection; 2] {
    [
        t::ParagraphBaseDirection::LeftToRight,
        t::ParagraphBaseDirection::RightToLeft,
    ]
}

fn interior(p: &t::PreparedText) -> Vec<t::PreparedOpportunity> {
    p.opportunity_views()
        .filter(|offer| offer.kind() != t::PreparedBreakKind::ParagraphEnd)
        .map(|offer| offer.handle())
        .collect()
}

fn qualify_shy(p: &t::PreparedText, source: &t::Source) -> t::PreparedBoundary {
    assert_eq!(source.text(), &t::Text::from("a\u{ad}b"));
    assert_eq!(source.spans().len(), 2);
    assert_eq!(p.analyzed_source().source(), source);
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        source.text()
    );
    assert!(
        p.participants()
            .all(|part| !matches!(part, t::PreparedParticipant::Control(_)))
    );
    let source_cuts = p
        .hyphenation_cut_views()
        .filter(|cut| cut.provenance() == t::HyphenationProvenance::Source)
        .collect::<Vec<_>>();
    assert_eq!(source_cuts.len(), 1);
    assert_eq!(source_cuts[0].style(), source.spans()[0].style());
    let cut = source_cuts[0].boundary();
    assert_eq!(cut.view().source_before().get(), 2);
    assert_eq!(cut.view().source_after().get(), 2);
    assert_eq!(cut.view().participant_offset(), 2);
    assert!(
        interior(p)
            .iter()
            .all(|offer| offer.view().boundary().view().source_before().get() != 1),
        "no cut before source SHY"
    );
    let whole = finish(p, start(p), end(p), None, 20.0);
    assert_eq!(whole.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&whole),
        [
            (t::Range::new(0, 1), 67, 10.0),
            (t::Range::new(2, 3), 68, 10.0)
        ]
    );
    copy_whole(p, whole, source);
    cut
}

fn shy_case(owner: t::WordBreak, following: t::WordBreak, ordinary: bool, manual: bool) {
    let mut observed = Vec::new();
    for direction in directions() {
        let (p, source) = shy_input(manual_style(owner), following, direction);
        let cut = qualify_shy(&p, &source);
        let offers = interior(&p);
        assert!(
            offers.iter().all(|offer| offer.view().boundary() == cut),
            "only the genuine source-after-SHY relation may be admitted"
        );
        let actual_ordinary = offers
            .iter()
            .any(|offer| offer.view().kind() == t::PreparedBreakKind::Allowed);
        let mut actual_manual = 0;
        for offer in &offers {
            if offer.view().kind() == t::PreparedBreakKind::Allowed {
                let line = finish(&p, start(&p), offer.clone(), None, 10.0);
                assert_eq!(line.feedback().advance, 10.0);
                assert_eq!(glyphs(&line), [(t::Range::new(0, 1), 67, 10.0)]);
            }
            for alternative in offer.view().alternatives() {
                assert_eq!(alternative.source_range(), Some(t::Range::new(1, 2)));
                assert_eq!(alternative.insertion_anchor().get(), 2);
                assert_eq!(alternative.text(), &t::Text::from("X"));
                let line = finish(
                    &p,
                    start(&p),
                    offer.clone(),
                    Some(alternative.handle()),
                    20.0,
                );
                assert_eq!(line.feedback().advance, 20.0);
                assert_eq!(
                    glyphs(&line),
                    [
                        (t::Range::new(0, 1), 67, 10.0),
                        (t::Range::new(2, 2), 58, 10.0)
                    ]
                );
                actual_manual += 1;
            }
        }
        assert_eq!(p.analyzed_source().source(), &source);
        observed.push((actual_ordinary, actual_manual));
    }
    assert_eq!(
        observed,
        vec![(ordinary, usize::from(manual)); 2],
        "(ordinary, manual count), LTR then RTL; owner={owner:?}, following={following:?}"
    );
}

#[test]
fn source_shy_break_all_owner_and_following_break_all_admit_only_real_ordinary_after_cut() {
    shy_case(t::WordBreak::BreakAll, t::WordBreak::BreakAll, true, false);
}

#[test]
fn source_shy_break_all_owner_and_following_normal_admit_no_cut() {
    shy_case(t::WordBreak::BreakAll, t::WordBreak::Normal, false, false);
}

#[test]
fn source_shy_normal_owner_and_following_break_all_keep_ordinary_and_manual_siblings() {
    shy_case(t::WordBreak::Normal, t::WordBreak::BreakAll, true, true);
}

#[test]
fn source_shy_normal_owner_and_following_normal_keep_manual_only() {
    shy_case(t::WordBreak::Normal, t::WordBreak::Normal, false, true);
}

#[test]
fn source_shy_disabled_hyphens_still_keeps_independent_break_all_after_permission() {
    let mut observed = Vec::new();
    for direction in directions() {
        let mut owner = manual_style(t::WordBreak::BreakAll);
        owner.hyphens = t::Hyphens::None;
        let (p, source) = shy_input(owner, t::WordBreak::BreakAll, direction);
        let cut = qualify_shy(&p, &source);
        let offers = interior(&p);
        for offer in &offers {
            assert_eq!(offer.view().boundary(), cut);
            assert_eq!(offer.view().kind(), t::PreparedBreakKind::Allowed);
            assert_eq!(offer.view().alternatives().count(), 0);
            let line = finish(&p, start(&p), offer.clone(), None, 10.0);
            assert_eq!(line.feedback().advance, 10.0);
            assert_eq!(glyphs(&line), [(t::Range::new(0, 1), 67, 10.0)]);
        }
        observed.push(offers.len());
    }
    assert_eq!(
        observed,
        [1, 1],
        "after-SHY ordinary permission survives hyphens:none in both directions"
    );
}

#[test]
fn empty_indication_remains_hyphenation_and_is_suppressed_by_break_all() {
    let mut observed = Vec::new();
    for direction in directions() {
        let mut owner = manual_style(t::WordBreak::BreakAll);
        owner.hyphenate_character = t::HyphenateCharacter::String(t::Text::from(""));
        let (p, source) = shy_input(owner, t::WordBreak::BreakAll, direction);
        qualify_shy(&p, &source);
        observed.push(p.alternative_views().count());
    }
    assert_eq!(
        observed,
        [0, 0],
        "empty painted indication does not authorize a hyphenation trial"
    );
}

fn control_input(
    kind: t::ParagraphControlKind,
    at: usize,
    text: &str,
    base: t::Style,
    direction: t::ParagraphBaseDirection,
) -> (t::PreparedText, t::Source) {
    let source = t::Source::identified(t::Text::from(text), t::Id::from_u64(65003), 19);
    let mut input = paragraph(source.clone(), base);
    input.base_direction = direction;
    input.controls.push(t::ParagraphControl { at, kind });
    input.anchor_order.push(t::ParagraphAnchor {
        item: t::ParagraphAnchorItem::Control(0),
        affinity: t::Affinity::After,
    });
    (prepare_input(input), source)
}

fn after_control(p: &t::PreparedText) -> t::PreparedBoundary {
    let participant = p
        .participants()
        .position(|part| matches!(part, t::PreparedParticipant::Control(_)))
        .unwrap();
    assert_eq!(
        p.participants()
            .filter(|part| matches!(part, t::PreparedParticipant::Control(_)))
            .count(),
        1
    );
    p.boundary_views()
        .find(|boundary| boundary.participant_offset() == participant + 1)
        .unwrap()
        .handle()
}

fn discretionary_case(owner: t::WordBreak, admitted: bool) {
    let mut observed = Vec::new();
    for direction in directions() {
        let (p, source) = control_input(
            t::ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(manual_style(owner)),
            },
            1,
            "ab",
            manual_style(t::WordBreak::Normal),
            direction,
        );
        assert_eq!(source.text(), &t::Text::from("ab"));
        assert_eq!(p.analyzed_source().source(), &source);
        let cut = after_control(&p);
        assert_eq!(cut.view().source_before().get(), 1);
        let cuts = p
            .hyphenation_cut_views()
            .filter(|row| row.provenance() == t::HyphenationProvenance::Control)
            .collect::<Vec<_>>();
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].boundary(), cut);
        let whole = finish(&p, start(&p), end(&p), None, 20.0);
        assert_eq!(whole.feedback().advance, 20.0);
        assert_eq!(
            glyphs(&whole),
            [
                (t::Range::new(0, 1), 67, 10.0),
                (t::Range::new(1, 2), 68, 10.0)
            ]
        );
        copy_whole(&p, whole, &source);
        let offers = interior(&p);
        for offer in &offers {
            assert_eq!(offer.view().boundary(), cut);
            assert_eq!(
                offer.view().kind(),
                t::PreparedBreakKind::Discretionary,
                "no BA character is invented for a source-free control"
            );
            for alt in offer.view().alternatives() {
                assert_eq!(alt.source_range(), None);
                let selected = finish(&p, start(&p), offer.clone(), Some(alt.handle()), 20.0);
                assert_eq!(selected.feedback().advance, 20.0);
                assert_eq!(
                    glyphs(&selected),
                    [
                        (t::Range::new(0, 1), 67, 10.0),
                        (t::Range::new(1, 1), 58, 10.0)
                    ]
                );
            }
        }
        observed.push((offers.len(), p.alternative_views().count()));
    }
    assert_eq!(
        observed,
        vec![(usize::from(admitted), usize::from(admitted)); 2],
        "actual discretionary control owner={owner:?}, both directions"
    );
}

#[test]
fn source_free_break_all_discretionary_control_has_no_manual_or_invented_ordinary_offer() {
    discretionary_case(t::WordBreak::BreakAll, false);
}

#[test]
fn source_free_normal_discretionary_control_keeps_actual_manual_offer() {
    discretionary_case(t::WordBreak::Normal, true);
}

#[test]
fn genuine_wbr_under_normal_and_keep_all_keeps_none_and_dictionary_auto_at_same_actual_endpoint() {
    for word_break in [t::WordBreak::Normal, t::WordBreak::KeepAll] {
        let base = style(word_break);
        let (p, source) = control_input(
            t::ParagraphControlKind::Wbr {
                style: Box::new(base.clone()),
            },
            2,
            WORD,
            base,
            t::ParagraphBaseDirection::LeftToRight,
        );
        assert_eq!(p.word_views().next().unwrap().query(), Some(WORD));
        assert_eq!(
            p.word_views().next().unwrap().provenance(),
            Some(t::HyphenationProvenance::SourceException { entry: WORD })
        );
        let actual = after_control(&p);
        assert_eq!(actual.view().source_before().get(), 2);
        let offer = p
            .opportunity_views()
            .find(|offer| offer.boundary() == actual)
            .unwrap()
            .handle();
        assert_eq!(offer.view().kind(), t::PreparedBreakKind::Allowed);
        let cut = p
            .hyphenation_cut_views()
            .find(|cut| {
                cut.boundary() == actual
                    && matches!(
                        cut.provenance(),
                        t::HyphenationProvenance::SourceException { entry: WORD }
                    )
            })
            .unwrap();
        assert_eq!(cut.opportunity(), Some(offer.clone()));
        let offer_view = offer.view();
        let alternatives = offer_view.alternatives().collect::<Vec<_>>();
        assert_eq!(alternatives.len(), 1);
        assert!(matches!(
            alternatives[0].definition(),
            t::PreparedAlternativeDefinition::Automatic { .. }
        ));
        let ordinary = finish(&p, start(&p), offer.clone(), None, 20.0);
        assert_eq!(ordinary.feedback().advance, 20.0);
        assert_eq!(glyphs(&ordinary), expected_letters(2));
        let generated = finish(
            &p,
            start(&p),
            offer.clone(),
            Some(alternatives[0].handle()),
            30.0,
        );
        assert_eq!(generated.feedback().advance, 30.0);
        let mut expected = expected_letters(2);
        expected.push((t::Range::new(2, 2), 58, 10.0));
        assert_eq!(glyphs(&generated), expected);
        assert_eq!(p.analyzed_source().source(), &source);
    }
}

#[test]
fn suppressed_source_shy_does_not_claim_an_admitted_manual_word_partition() {
    let (p, source) = shy_input(
        manual_style(t::WordBreak::BreakAll),
        t::WordBreak::BreakAll,
        t::ParagraphBaseDirection::LeftToRight,
    );
    qualify_shy(&p, &source);
    let portions = p.word_views().next().unwrap().portions().unwrap();
    assert!(
        !portions.is_empty(),
        "a suppressed manual cut retains a complete measurement word"
    );
    assert!(
        portions
            .iter()
            .all(|portion| !portion.has_manual_partition()),
        "a truthful Source CutData or an Allowed-only offer is not an admitted manual alternative"
    );
}

#[test]
fn suppressed_source_shy_retains_whole_word_portion_and_measurement_tail() {
    let (p, source) = shy_input(
        manual_style(t::WordBreak::BreakAll),
        t::WordBreak::BreakAll,
        t::ParagraphBaseDirection::LeftToRight,
    );
    qualify_shy(&p, &source);
    let portions = p.word_views().next().unwrap().portions().unwrap();
    assert_eq!(
        portions.len(),
        1,
        "no eligible manual ending: keep one whole word portion"
    );
    let portion = &portions[0];
    assert_eq!(portion.start(), start(&p));
    assert_eq!(portion.end(), end(&p).view().boundary());
    assert!(portion.incoming().is_none());
    assert!(portion.alternative().is_none());
    let from = portion.start().view().participant_offset();
    let to = portion.end().view().participant_offset();
    let measured = p
        .select(t::LineSelectionRequest {
            inline_boxes: None,
            segment: p.source_segments().next().unwrap(),
            start: portion.start(),
            end: portion.end(),
            participants: p.participants().skip(from).take(to - from).collect(),
            termination: t::PreparedTermination::Measurement(portion.clone()),
            alternative: portion.alternative(),
            incoming: portion.incoming(),
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_box_struts: Vec::new(),
            line_strut: None,
            context: t::LineContext {
                alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
                formatted_line: surgeist_text::FormattedLineContext::default(),
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                line_box_direction: surgeist_text::LineBoxDirection::LeftToRight,
                orientation: t::LineOrientation::Horizontal,
                start: t::LineStart::Paragraph,
                end: t::LineEnd::Soft,
                available_inline: Some(20.0),
                indent: 0.0,
                alignment: t::Alignment::Start,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(measured.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&measured),
        [
            (t::Range::new(0, 1), 67, 10.0),
            (t::Range::new(2, 3), 68, 10.0)
        ]
    );
    assert_eq!(p.analyzed_source().source(), &source);
}

#[test]
fn genuine_wbr_keeps_ordinary_permission_when_break_all_suppresses_its_auto_sibling() {
    let mut observed = Vec::new();
    for direction in directions() {
        let base = style(t::WordBreak::BreakAll);
        let (p, source) = control_input(
            t::ParagraphControlKind::Wbr {
                style: Box::new(base.clone()),
            },
            2,
            WORD,
            base,
            direction,
        );
        let actual = after_control(&p);
        let offer = p
            .opportunity_views()
            .find(|offer| offer.boundary() == actual)
            .unwrap()
            .handle();
        assert_eq!(offer.view().kind(), t::PreparedBreakKind::Allowed);
        let ordinary = finish(&p, start(&p), offer.clone(), None, 20.0);
        assert_eq!(ordinary.feedback().advance, 20.0);
        assert_eq!(glyphs(&ordinary), expected_letters(2));
        assert_eq!(p.analyzed_source().source(), &source);
        observed.push(offer.view().alternatives().count());
    }
    assert_eq!(
        observed,
        [0, 0],
        "genuine Wbr None sibling remains, Auto is suppressed"
    );
}

#[test]
fn break_all_preserves_independent_zwsp_forced_and_visible_hyphen_permissions() {
    for direction in directions() {
        for (text, kind, expected) in [
            (
                "a\u{200b}b",
                t::PreparedBreakKind::Allowed,
                vec![(t::Range::new(0, 1), 67, 10.0)],
            ),
            (
                "a\nb",
                t::PreparedBreakKind::Forced,
                vec![(t::Range::new(0, 1), 67, 10.0)],
            ),
            (
                "a-b",
                t::PreparedBreakKind::Allowed,
                vec![
                    (t::Range::new(0, 1), 67, 10.0),
                    (t::Range::new(1, 2), 15, 10.0),
                ],
            ),
        ] {
            let mut base = manual_style(t::WordBreak::BreakAll);
            base.hyphens = t::Hyphens::None;
            let source = t::Source::identified(t::Text::from(text), t::Id::from_u64(65004), 23);
            let mut input = paragraph(source.clone(), base);
            input.base_direction = direction;
            let p = prepare_input(input);
            let actual = p
                .opportunity_views()
                .find(|offer| offer.boundary().view().source_before().get() == 2)
                .unwrap()
                .handle();
            assert_eq!(actual.view().kind(), kind);
            assert_eq!(actual.view().alternatives().count(), 0);
            assert!(
                !p.opportunity_views()
                    .any(|offer| offer.boundary().view().source_before().get() == 1)
            );
            let prefix = finish(&p, start(&p), actual, None, width(expected.len()));
            assert_eq!(prefix.feedback().advance, width(expected.len()));
            assert_eq!(glyphs(&prefix), expected);
            assert_eq!(p.analyzed_source().source(), &source);
            if text == "a-b" {
                assert_eq!(
                    skrifa::FontRef::new(AHEM)
                        .unwrap()
                        .charmap()
                        .map('-')
                        .unwrap()
                        .to_u32(),
                    15
                );
            }
        }
    }
}

#[test]
fn break_all_preserves_real_atomic_before_and_after_permissions_without_hyphenation() {
    for direction in directions() {
        let atomic = t::InlineBox::new(
            t::Id::from_u64(65005),
            t::InlineBoxKind::InFlow,
            1,
            t::Size::new(3.0, 4.0),
        );
        let mut source = t::Source::identified(t::Text::from("ab"), t::Id::from_u64(65006), 29);
        source.inline_box(atomic);
        let mut base = manual_style(t::WordBreak::BreakAll);
        base.hyphens = t::Hyphens::None;
        let mut input = paragraph(source.clone(), base);
        input.base_direction = direction;
        input.anchor_order.push(t::ParagraphAnchor {
            item: t::ParagraphAnchorItem::Atomic(0),
            affinity: t::Affinity::After,
        });
        for definition in [
            t::ParagraphBoundaryDefinition::AtomicBefore(0),
            t::ParagraphBoundaryDefinition::AtomicAfter(0),
        ] {
            input.boundary_policies.push(t::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: t::Wrap::Word,
                following_wrap: t::Wrap::Word,
                ancestor_wrap: t::Wrap::Word,
            });
        }
        let p = prepare_input(input);
        let issued = p.atomic_views().next().unwrap().handle();
        assert_eq!(p.atomic_views().count(), 1);
        let at = p.participants().position(|part| matches!(part, t::PreparedParticipant::Atomic(ref handle) if *handle == issued)).unwrap();
        for offset in [at, at + 1] {
            let actual = p
                .opportunity_views()
                .find(|offer| offer.boundary().view().participant_offset() == offset)
                .unwrap();
            assert_eq!(actual.kind(), t::PreparedBreakKind::Allowed);
            assert_eq!(actual.boundary().view().source_before().get(), 1);
            assert_eq!(actual.alternatives().count(), 0);
        }
        assert_eq!(p.alternative_views().count(), 0);
        assert_eq!(p.analyzed_source().source(), &source);
        assert_eq!(source.boxes(), &[atomic]);
    }
}

#[test]
fn coincident_wbr_cannot_transfer_ordinary_authority_to_suppressed_discretionary_control() {
    let mut observed = Vec::new();
    for direction in directions() {
        let source = t::Source::identified(t::Text::from("ab"), t::Id::from_u64(65007), 31);
        let mut input = paragraph(source.clone(), manual_style(t::WordBreak::Normal));
        input.base_direction = direction;
        input.controls = vec![
            t::ParagraphControl {
                at: 1,
                kind: t::ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(manual_style(t::WordBreak::BreakAll)),
                },
            },
            t::ParagraphControl {
                at: 1,
                kind: t::ParagraphControlKind::Wbr {
                    style: Box::new(manual_style(t::WordBreak::Normal)),
                },
            },
        ];
        input.anchor_order = (0..2)
            .map(|index| t::ParagraphAnchor {
                item: t::ParagraphAnchorItem::Control(index),
                affinity: t::Affinity::After,
            })
            .collect();
        let p = prepare_input(input);
        let controls = p
            .participants()
            .enumerate()
            .filter_map(|(index, part)| {
                matches!(part, t::PreparedParticipant::Control(_)).then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(controls.len(), 2);
        let actual_after = controls
            .iter()
            .map(|&index| {
                p.boundary_views()
                    .find(|boundary| boundary.participant_offset() == index + 1)
                    .unwrap()
                    .handle()
            })
            .collect::<Vec<_>>();
        assert_ne!(actual_after[0], actual_after[1]);
        for cut in &actual_after {
            assert_eq!(cut.view().source_before().get(), 1);
            assert_eq!(cut.view().source_after().get(), 1);
        }
        let wbr = p
            .opportunity_views()
            .find(|offer| offer.boundary() == actual_after[1])
            .unwrap()
            .handle();
        assert_eq!(wbr.view().kind(), t::PreparedBreakKind::Allowed);
        assert_eq!(wbr.view().alternatives().count(), 0);
        let line = finish(&p, start(&p), wbr, None, 10.0);
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(glyphs(&line), [(t::Range::new(0, 1), 67, 10.0)]);
        let cut = p
            .hyphenation_cut_views()
            .find(|cut| cut.provenance() == t::HyphenationProvenance::Control)
            .unwrap();
        assert_eq!(cut.boundary(), actual_after[0]);
        observed.push(
            p.opportunity_views()
                .filter(|offer| offer.boundary() == actual_after[0])
                .count(),
        );
        assert_eq!(p.analyzed_source().source(), &source);
    }
    assert_eq!(
        observed,
        [0, 0],
        "equal source offsets do not transfer the Wbr's actual control permission"
    );
}

// An in-flow atomic is the actual influencing neighbor at each incident cut.
// Neither source SHY nor the text beyond the atomic can replace that authority.
fn shy_atomic_input(
    owner: t::WordBreak,
    following: t::WordBreak,
    direction: t::ParagraphBaseDirection,
    before_ancestor: t::Wrap,
    after_ancestor: t::Wrap,
) -> (t::PreparedText, t::Source, [t::PreparedBoundary; 2]) {
    let atomic = t::InlineBox::new(
        t::Id::from_u64(65008),
        t::InlineBoxKind::InFlow,
        2,
        t::Size::new(3.0, 4.0),
    );
    let mut base = manual_style(owner);
    base.hyphens = t::Hyphens::None;
    let mut source = t::Source::identified(t::Text::from("a\u{ad}b"), t::Id::from_u64(65009), 37);
    source.span(t::Range::new(1, 2), base.clone());
    let mut next = base.clone();
    next.word_break = following;
    source.span(t::Range::new(2, 3), next);
    source.inline_box(atomic);
    let mut input = paragraph(source.clone(), base.clone());
    input.base_direction = direction;
    input.anchor_order.push(t::ParagraphAnchor {
        item: t::ParagraphAnchorItem::Atomic(0),
        affinity: t::Affinity::After,
    });
    for (definition, ancestor_wrap) in [
        (
            t::ParagraphBoundaryDefinition::AtomicBefore(0),
            before_ancestor,
        ),
        (
            t::ParagraphBoundaryDefinition::AtomicAfter(0),
            after_ancestor,
        ),
    ] {
        input.boundary_policies.push(t::ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: t::Wrap::Word,
            following_wrap: t::Wrap::Word,
            ancestor_wrap,
        });
    }
    let p = prepare_input(input);
    assert_eq!(p.analyzed_source().source(), &source);
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        source.text()
    );
    assert_eq!(source.text(), &t::Text::from("a\u{ad}b"));
    assert_eq!(source.boxes(), &[atomic]);
    assert_eq!(
        p.control_views().count(),
        0,
        "no synthetic SHY control or BA character"
    );
    assert_eq!(p.alternative_views().count(), 0);
    let atomics = p.atomic_views().collect::<Vec<_>>();
    assert_eq!(atomics.len(), 1);
    assert_eq!(atomics[0].original_box(), &atomic);
    assert_eq!(atomics[0].source_position().get(), 2);
    let issued = atomics[0].handle();
    let at = p
        .participants()
        .position(
            |part| matches!(part, t::PreparedParticipant::Atomic(ref handle) if *handle == issued),
        )
        .unwrap();
    assert_eq!(at, 2);
    let boundaries = [at, at + 1].map(|offset| {
        p.boundary_views()
            .find(|boundary| boundary.participant_offset() == offset)
            .unwrap()
            .handle()
    });
    assert_ne!(boundaries[0], boundaries[1]);
    for boundary in &boundaries {
        assert_eq!(boundary.view().source_before().get(), 2);
        assert_eq!(boundary.view().source_after().get(), 2);
    }
    let source_cuts = p
        .hyphenation_cut_views()
        .filter(|cut| cut.provenance() == t::HyphenationProvenance::Source)
        .collect::<Vec<_>>();
    assert_eq!(source_cuts.len(), 1);
    assert_eq!(source_cuts[0].boundary(), boundaries[0]);
    assert_eq!(source_cuts[0].style(), &base);
    assert!(
        !p.opportunity_views()
            .any(|offer| offer.boundary().view().source_before().get() == 1),
        "no before-SHY permission"
    );
    (p, source, boundaries)
}

#[test]
fn inactive_source_shy_preserves_both_actual_atomic_permissions_without_borrowing_far_text() {
    let mut observed = Vec::new();
    for direction in directions() {
        for owner in [t::WordBreak::Normal, t::WordBreak::BreakAll] {
            for following in [t::WordBreak::Normal, t::WordBreak::BreakAll] {
                let (p, source, boundaries) =
                    shy_atomic_input(owner, following, direction, t::Wrap::Word, t::Wrap::Word);
                let actual = boundaries.map(|boundary| {
                    let offer = p
                        .opportunity_views()
                        .find(|offer| offer.boundary() == boundary);
                    if let Some(offer) = &offer {
                        assert_eq!(offer.alternatives().count(), 0);
                    }
                    offer.map(|offer| offer.handle())
                });
                if let Some(before) = &actual[0] {
                    // AtomicBefore excludes the actual atomic: only a and the
                    // unpainted authored SHY participate. No atomic allocation.
                    let prefix = finish(&p, start(&p), before.clone(), None, 10.0);
                    assert_eq!(prefix.feedback().advance, 10.0);
                    assert_eq!(glyphs(&prefix), [(t::Range::new(0, 1), 67, 10.0)]);
                    assert!(prefix.selected_line().request().atomics.is_empty());
                }
                observed.push((
                    direction,
                    owner,
                    following,
                    actual.map(|offer| offer.map(|offer| offer.view().kind())),
                ));
                assert_eq!(p.analyzed_source().source(), &source);
            }
        }
    }
    assert!(
        observed
            .iter()
            .all(|row| row.3 == [Some(t::PreparedBreakKind::Allowed); 2]),
        "actual AtomicBefore/After Word permission, independent of source SHY/far following text: {observed:?}"
    );
}

#[test]
fn inactive_source_shy_atomic_permissions_obey_each_actual_ancestor_and_do_not_borrow_far_break_all()
 {
    let mut observed = Vec::new();
    for direction in directions() {
        for ancestors in [
            [t::Wrap::None, t::Wrap::None],
            [t::Wrap::None, t::Wrap::Word],
            [t::Wrap::Word, t::Wrap::None],
        ] {
            let (p, source, boundaries) = shy_atomic_input(
                t::WordBreak::BreakAll,
                t::WordBreak::BreakAll,
                direction,
                ancestors[0],
                ancestors[1],
            );
            let actual = boundaries.map(|boundary| {
                p.opportunity_views()
                    .find(|offer| offer.boundary() == boundary)
                    .map(|offer| {
                        assert_eq!(offer.alternatives().count(), 0);
                        offer.kind()
                    })
            });
            let expected = ancestors.map(|ancestor| {
                (ancestor == t::Wrap::Word).then_some(t::PreparedBreakKind::Allowed)
            });
            observed.push((direction, ancestors, actual, expected));
            assert_eq!(p.analyzed_source().source(), &source);
        }
    }
    assert!(
        observed.iter().all(|row| row.2 == row.3),
        "each genuine atomic NCA owns only its own permission: {observed:?}"
    );
}
