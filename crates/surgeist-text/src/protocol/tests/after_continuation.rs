//! Independent functional oracles for continuation custody and joint ownership.
#[path = "both_sides.rs"]
mod both_sides;
#[path = "casing_indications.rs"]
mod casing_indications;
#[path = "incoming_custody.rs"]
mod incoming_custody;
#[path = "wbr_evidence.rs"]
mod wbr_evidence;
#[path = "width_indications.rs"]
mod width_indications;
use super::*;
use crate::TextSeam;
use crate::{
    HyphenateCharacter, HyphenationCounts, HyphenationError, HyphenationPlacement, Hyphens,
    LineHeight, NativeHyphenationConvention, Range, SourceRangeFragmentError, Text,
};
use skrifa::MetadataProvider;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn own_style(text: &str, placement: HyphenationPlacement) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        hyphens: Hyphens::Manual,
        locale: Some("und-Latn".into()),
        hyphenate_character: HyphenateCharacter::Auto,
        native_hyphenation_convention: Some(
            NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                Text::from(text),
                placement,
            )
            .unwrap(),
        ),
        ..Style::default()
    }
}

fn capture(source: Source, base_style: Style, bytes: &[u8]) -> PreparedText {
    let length = source.text().len();
    let anchor_order = (0..source.boxes().len())
        .map(|occurrence| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Atomic(occurrence),
            affinity: crate::Affinity::After,
        })
        .collect();
    let input = ParagraphInput {
        source: source.clone(),
        base_style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order,
    };
    capture_input(input, bytes)
}
fn capture_input(input: ParagraphInput, bytes: &[u8]) -> PreparedText {
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn start(p: &PreparedText) -> PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}
fn end(p: &PreparedText) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}
fn cut(p: &PreparedText, at: usize, counts: HyphenationCounts) -> PreparedOpportunity {
    let c = p
        .hyphenation_cut_views()
        .find(|v| v.boundary().view().source_before().get() == at)
        .expect("actual source SHY cut must be issued");
    assert_eq!(c.counts(), Some(counts));
    let offer = c
        .opportunity()
        .expect("actual native counts must admit the cut");
    assert_eq!(offer.view().kind(), PreparedBreakKind::Discretionary);
    assert_eq!(offer.view().alternatives().count(), 1);
    offer
}
fn alternative(offer: &PreparedOpportunity) -> PreparedAlternative {
    offer.view().alternatives().next().unwrap().handle()
}
fn selected_request(
    p: &PreparedText,
    from: PreparedBoundary,
    offer: PreparedOpportunity,
    incoming: Option<HyphenationContinuation>,
    take_alternative: bool,
) -> LineSelectionRequest {
    let mut r = request(p);
    r.start = from;
    r.end = offer.view().boundary();
    r.incoming = incoming;
    r.alternative = take_alternative.then(|| alternative(&offer));
    r.termination = PreparedTermination::Ordinary(offer.clone());
    let first = r.start.view().participant_offset();
    let last = r.end.view().participant_offset();
    r.participants = p.participants().skip(first).take(last - first).collect();
    r.controls = r
        .participants
        .iter()
        .filter_map(|part| {
            if let PreparedParticipant::Control(occurrence) = part {
                Some({
                    let occurrence = occurrence.clone();
                    ControlAllocation {
                        boundary: matches!(
                            occurrence.view().kind(),
                            crate::ParagraphControlKind::InlineBoundaryStart
                                | crate::ParagraphControlKind::InlineBoundaryEnd
                        )
                        .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                        occurrence,
                        strut: None,
                    }
                })
            } else {
                None
            }
        })
        .collect();
    r.context.start = if first == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = if offer.view().kind() == PreparedBreakKind::ParagraphEnd {
        LineEnd::Paragraph
    } else {
        LineEnd::Soft
    };
    r.context.spacing = SpacingPolicy::None;
    r.context.available_inline = Some(100.0);
    r
}
fn finish(p: &PreparedText, r: LineSelectionRequest) -> crate::Result<FinalizedLine> {
    p.select(r)?.finalize(crate::LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: crate::ScopeAllocations::None,
    })
}
fn place_lines(
    p: &PreparedText,
    coverage: PlacementCoverage,
    lines: Vec<FinalizedLine>,
) -> crate::Result<crate::Layout> {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                let baseline = 8.0 + 10.0 * index as f64;
                let atomics = line
                    .atomic_slots()
                    .map(|slot| {
                        assert_eq!(slot.allocation, AtomicLineAllocation::OutOfFlow);
                        assert_eq!(
                            slot.occurrence.view().original_box().size(),
                            crate::Size::new(0.0, 0.0)
                        );
                        let point = crate::Point::new(slot.origin.x, baseline + slot.origin.y);
                        AtomicPlacement {
                            occurrence: slot.occurrence,
                            border: crate::Rect::new(point.x, point.y, 0.0, 0.0),
                            content: crate::Rect::new(point.x, point.y, 0.0, 0.0),
                            baseline: point.y,
                        }
                    })
                    .collect();
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics,
                })
            })
            .collect(),
    })
}
fn hyphenation_reason(error: &crate::Error, expected: HyphenationError) {
    assert!(
        matches!(error.detail(), Some(crate::ErrorDetail::Hyphenation { reason }) if *reason == expected),
        "{error:?}"
    );
}

#[derive(Debug, PartialEq)]
struct Glyph {
    id: u32,
    advance: f32,
    x: f64,
    y: f32,
}
fn glyphs(line: &FinalizedLine, bytes: &[u8]) -> Vec<Glyph> {
    let source = line
        .prepared_text()
        .analyzed_source()
        .source()
        .text()
        .code_units()
        .to_vec();
    let mut out = vec![];
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 10.0);
        for glyph in run.glyphs() {
            let r = glyph.range();
            // Only an actual authored SHY formatting row may be omitted here.
            // Generated zero-advance marks and controls are not filtered by width.
            if glyph.advance() == 0.0
                && matches!(glyph.id(), 0 | 3)
                && r.end == r.start + 1
                && source.get(r.start) == Some(&0xad)
            {
                continue;
            }
            out.push(Glyph {
                id: glyph.id(),
                advance: glyph.advance(),
                x: run.origin().x + glyph.pen() + f64::from(glyph.x()),
                y: glyph.y(),
            });
        }
    }
    out
}
fn counts(before: usize, after: usize) -> HyphenationCounts {
    HyphenationCounts {
        total: before + after,
        before,
        after,
    }
}
fn plain_pair() -> (PreparedText, FinalizedLine, FinalizedLine) {
    let p = capture(
        Source::identified("AB\u{ad}CDE", crate::Id::from_u64(66190), 4),
        own_style("A", HyphenationPlacement::LogicalAfter),
        AHEM,
    );
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 66, 173, 67, 68, 69]
    );
    let offer = cut(&p, 3, counts(2, 3));
    let first = finish(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
    )
    .unwrap();
    let token = first.hyphenation_continuation().unwrap();
    assert_eq!(token.alternative(), alternative(&offer));
    assert_eq!(token.boundary(), offer.view().boundary());
    let second = finish(
        &p,
        selected_request(&p, offer.view().boundary(), end(&p), Some(token), false),
    )
    .unwrap();
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (20.0, 40.0)
    );
    assert_eq!(
        glyphs(&first, AHEM),
        [
            Glyph {
                id: 35,
                advance: 10.0,
                x: 0.0,
                y: 0.0
            },
            Glyph {
                id: 36,
                advance: 10.0,
                x: 10.0,
                y: 0.0
            }
        ]
    );
    assert_eq!(
        glyphs(&second, AHEM),
        [
            Glyph {
                id: 35,
                advance: 10.0,
                x: 0.0,
                y: 0.0
            },
            Glyph {
                id: 37,
                advance: 10.0,
                x: 10.0,
                y: 0.0
            },
            Glyph {
                id: 38,
                advance: 10.0,
                x: 20.0,
                y: 0.0
            },
            Glyph {
                id: 39,
                advance: 10.0,
                x: 30.0,
                y: 0.0
            }
        ]
    );
    (p, first, second)
}

#[test]
fn exact_whole_pair_and_partial_ranges_keep_source_query_coverage_separate_from_incoming_ownership()
{
    let (p, first, second) = plain_pair();
    let origin = p.origin_views().next().unwrap().handle();
    let whole = place_lines(
        &p,
        PlacementCoverage::WholeParagraph,
        vec![first.clone(), second.clone()],
    )
    .unwrap();
    let selected_shy = whole
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    let units: Vec<_> = selected_shy.units().collect();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].line_index(), 1);
    assert_eq!(
        units[0].alternatives().collect::<Vec<_>>(),
        [first.hyphenation_continuation().unwrap().alternative()]
    );
    assert_eq!(
        units[0]
            .defining_sources()
            .map(|v| v.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(2, 3)]
    );
    assert!(units[0].hyphenation_cuts().next().is_none());
    assert_eq!(
        units[0].pieces().map(|v| v.rect()).collect::<Vec<_>>(),
        [crate::Rect::new(0.0, 10.0, 10.0, 10.0)]
    );
    let receiver = place_lines(
        &p,
        PlacementCoverage::Range {
            start: second.selected_line().request().start.clone(),
            end: second.selected_line().request().end.clone(),
        },
        vec![second],
    )
    .unwrap();
    let error = receiver
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap_err();
    assert!(
        matches!(error.detail(), Some(crate::ErrorDetail::SourceRangeFragments {
        reason: SourceRangeFragmentError::OutsidePlacement { aggregate }
    }) if *aggregate == Range::new(2,3))
    );
    let c = receiver
        .source_range_fragments(&origin, Range::new(3, 4))
        .unwrap();
    assert!(c.units().all(|unit| unit.alternatives().next().is_none()));
    assert_eq!(
        c.units()
            .flat_map(|unit| unit.pieces().map(|v| v.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [crate::Rect::new(10.0, 0.0, 10.0, 10.0)]
    );
    let ending = place_lines(
        &p,
        PlacementCoverage::Range {
            start: first.selected_line().request().start.clone(),
            end: first.selected_line().request().end.clone(),
        },
        vec![first.clone()],
    )
    .unwrap();
    assert_eq!(ending.lines().len(), 1);
    assert!(
        ending
            .source_range_fragments(&origin, Range::new(2, 3))
            .unwrap()
            .units()
            .next()
            .is_none()
    );
    assert!(first.hyphenation_continuation().is_some());
}

#[test]
fn missing_foreign_and_same_text_different_cut_continuations_cannot_stitch_a_whole_paragraph() {
    let (p, first, second) = plain_pair();
    let mut absent = second.selected_line().request().clone();
    absent.incoming = None;
    let absent = finish(&p, absent).unwrap();
    hyphenation_reason(
        &place_lines(
            &p,
            PlacementCoverage::WholeParagraph,
            vec![first.clone(), absent],
        )
        .unwrap_err(),
        HyphenationError::InvalidContinuation,
    );
    let (foreign, _, foreign_second) = plain_pair();
    assert_ne!(p, foreign);
    let mut wrong = foreign_second.selected_line().request().clone();
    wrong.incoming = first.hyphenation_continuation();
    hyphenation_reason(
        &foreign.select(wrong).unwrap_err(),
        HyphenationError::InvalidContinuation,
    );

    let same = capture(
        Source::new("AB\u{ad}CD\u{ad}EF"),
        own_style("A", HyphenationPlacement::LogicalAfter),
        AHEM,
    );
    let one = cut(&same, 3, counts(2, 4));
    let two = cut(&same, 6, counts(4, 2));
    let a = finish(
        &same,
        selected_request(&same, start(&same), one.clone(), None, true),
    )
    .unwrap();
    let b = finish(
        &same,
        selected_request(
            &same,
            one.view().boundary(),
            two.clone(),
            a.hyphenation_continuation(),
            true,
        ),
    )
    .unwrap();
    let ta = a.hyphenation_continuation().unwrap();
    let tb = b.hyphenation_continuation().unwrap();
    assert_eq!(
        ta.alternative().view().text(),
        tb.alternative().view().text()
    );
    assert_ne!(ta.alternative(), tb.alternative());
    let wrong_cut = selected_request(&same, one.view().boundary(), two.clone(), Some(tb), true);
    hyphenation_reason(
        &same.select(wrong_cut).unwrap_err(),
        HyphenationError::InvalidContinuation,
    );
    let duplicate = selected_request(&same, two.view().boundary(), end(&same), Some(ta), false);
    hyphenation_reason(
        &same.select(duplicate).unwrap_err(),
        HyphenationError::InvalidContinuation,
    );
}

#[test]
fn actual_manual_portions_retain_incoming_choice_and_measure_its_complete_contribution() {
    let (p, first, _) = plain_pair();
    let portions = p.word_views().next().unwrap().portions().unwrap();
    assert_eq!(portions.len(), 2);
    assert!(portions[0].incoming().is_none());
    assert_eq!(portions[1].incoming(), first.hyphenation_continuation());
    for (portion, expected) in portions.into_iter().zip([20.0, 40.0]) {
        let mut r = request(&p);
        r.start = portion.start();
        r.end = portion.end();
        r.incoming = portion.incoming();
        r.alternative = portion.alternative();
        r.termination = PreparedTermination::Measurement(portion);
        let from = r.start.view().participant_offset();
        let to = r.end.view().participant_offset();
        r.participants = p.participants().skip(from).take(to - from).collect();
        r.context.start = if from == 0 {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        r.context.end = LineEnd::Soft;
        r.context.available_inline = None;
        r.context.spacing = SpacingPolicy::None;
        let line = finish(&p, r).unwrap();
        assert_eq!(line.feedback().advance, expected);
        assert!(matches!(
            line.selected_line().request().termination,
            PreparedTermination::Measurement(_)
        ));
        assert!(
            place_lines(&p, PlacementCoverage::WholeParagraph, vec![line]).is_err(),
            "a portion measurement never acquires placement authority"
        );
    }
}

fn simultaneous(text: &str, suffix: &str, bytes: &[u8]) -> (PreparedText, Vec<FinalizedLine>) {
    let mut source = Source::new(text);
    source.span(
        Range::new(3, 4),
        own_style(suffix, HyphenationPlacement::LogicalBefore),
    );
    let p = capture(
        source,
        own_style("A", HyphenationPlacement::LogicalAfter),
        bytes,
    );
    let first_offer = cut(&p, 3, counts(2, 3));
    let second_offer = cut(&p, 4, counts(2, 3));
    assert_ne!(
        first_offer.view().boundary(),
        second_offer.view().boundary()
    );
    let first = finish(
        &p,
        selected_request(&p, start(&p), first_offer.clone(), None, true),
    )
    .unwrap();
    let middle = finish(
        &p,
        selected_request(
            &p,
            first_offer.view().boundary(),
            second_offer.clone(),
            first.hyphenation_continuation(),
            true,
        ),
    )
    .unwrap();
    assert!(
        middle.hyphenation_continuation().is_none(),
        "outgoing Before creates no new incoming obligation"
    );
    let last = finish(
        &p,
        selected_request(&p, second_offer.view().boundary(), end(&p), None, false),
    )
    .unwrap();
    (p, vec![first, middle, last])
}

#[test]
fn simultaneous_after_and_before_share_real_aa_substitution_without_merging_range_egcs() {
    let bytes = crate::shaping_test_fonts::ligature_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map(0x41_u32).unwrap().to_u32(), 35);
    let (p, lines) = simultaneous("XX\u{ad}\u{ad}XXX", "A", &bytes);
    assert_eq!(
        lines
            .iter()
            .map(|v| v.feedback().advance)
            .collect::<Vec<_>>(),
        [20.0, 10.0, 30.0]
    );
    assert_eq!(
        glyphs(&lines[1], &bytes),
        [Glyph {
            id: 58,
            advance: 10.0,
            x: 0.0,
            y: 0.0
        }]
    );
    let incoming = lines[0].hyphenation_continuation().unwrap().alternative();
    let outgoing = lines[1]
        .selected_line()
        .request()
        .alternative
        .clone()
        .unwrap();
    let participant = p
        .unit_views()
        .find(|v| v.source_range() == Range::new(3, 4))
        .unwrap()
        .handle();
    let seam = TextSeam::BeforeParticipant(PreparedParticipant::Text(participant));
    let prefix = lines[1].prefix_at(&seam).unwrap();
    let joint = prefix
        .interior_unit()
        .expect("actual participant seam lies between jointly substituted A atoms");
    let owners = joint.alternatives().collect::<Vec<_>>();
    assert_eq!(owners, [incoming.clone(), outgoing.clone()]);
    assert_eq!(joint.advance().unwrap(), 10.0);
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, lines).unwrap();
    let left = layout
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    let right = layout
        .source_range_fragments(&origin, Range::new(3, 4))
        .unwrap();
    let a = left.units().next().unwrap();
    let b = right.units().next().unwrap();
    assert!(
        !a.same_unit(&b),
        "provider ligature does not erase the two genuine EGCs"
    );
    assert_eq!(a.alternatives().collect::<Vec<_>>(), [incoming]);
    assert_eq!(b.alternatives().collect::<Vec<_>>(), [outgoing]);
    assert_eq!(a.pieces().map(|v| v.rect().size.width).sum::<f64>(), 5.0);
    assert_eq!(b.pieces().map(|v| v.rect().size.width).sum::<f64>(), 5.0);
}

#[test]
fn simultaneous_after_and_before_share_a_mark_egc_and_both_source_definitions() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map(0x301_u32).unwrap().to_u32(), 100);
    // Original A+acute supplies an authentic complete query witness; two partial
    // captured queries alone cannot certify the combined generated EGC.
    let (p, lines) = simultaneous("XX\u{ad}\u{ad}A\u{301}XX", "\u{301}", &bytes);
    assert_eq!(lines[1].feedback().advance, 10.0);
    assert_eq!(
        glyphs(&lines[1], &bytes),
        [
            Glyph {
                id: 35,
                advance: 10.0,
                x: 0.0,
                y: 0.0
            },
            Glyph {
                id: 100,
                advance: 0.0,
                x: 3.0,
                y: -4.0
            }
        ]
    );
    let incoming = lines[0].hyphenation_continuation().unwrap().alternative();
    let outgoing = lines[1]
        .selected_line()
        .request()
        .alternative
        .clone()
        .unwrap();
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, lines).unwrap();
    let left = layout
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    let right = layout
        .source_range_fragments(&origin, Range::new(3, 4))
        .unwrap();
    let a = left.units().next().unwrap();
    let b = right.units().next().unwrap();
    assert!(a.same_unit(&b));
    assert_eq!(a.alternatives().collect::<Vec<_>>(), [incoming, outgoing]);
    assert_eq!(
        a.defining_sources()
            .map(|v| v.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(2, 3), Range::new(3, 4)]
    );
    assert!(a.hyphenation_cuts().next().is_none());
    assert_eq!(a.pieces().map(|v| v.rect().size.width).sum::<f64>(), 10.0);
}

#[test]
fn simultaneous_after_and_before_keep_cross_egc_pair_positioning() {
    let bytes = crate::shaping_test_fonts::pair_adjusted_ahem();
    let (p, lines) = simultaneous("XX\u{ad}\u{ad}XXX", "B", &bytes);
    assert_eq!(lines[1].feedback().advance, 15.0);
    assert_eq!(
        glyphs(&lines[1], &bytes),
        [
            Glyph {
                id: 35,
                advance: 5.0,
                x: 0.0,
                y: 0.0
            },
            Glyph {
                id: 36,
                advance: 10.0,
                x: 5.0,
                y: 0.0
            }
        ]
    );
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place_lines(&p, PlacementCoverage::WholeParagraph, lines).unwrap();
    assert_eq!(layout.lines().len(), 3);
    assert_eq!(
        layout
            .source_range_fragments(&origin, Range::new(2, 3))
            .unwrap()
            .units()
            .next()
            .unwrap()
            .pieces()
            .map(|v| v.rect().size.width)
            .sum::<f64>(),
        5.0
    );
}

#[test]
fn a_covering_face_and_two_partial_queries_cannot_certify_the_combined_mark_query() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let mut source = Source::new("XX\u{ad}\u{ad}BXX");
    source.span(
        Range::new(3, 4),
        own_style("\u{301}", HyphenationPlacement::LogicalBefore),
    );
    let p = capture(
        source,
        own_style("A", HyphenationPlacement::LogicalAfter),
        &bytes,
    );
    let one = cut(&p, 3, counts(2, 3));
    let two = cut(&p, 4, counts(2, 3));
    let first = finish(&p, selected_request(&p, start(&p), one.clone(), None, true)).unwrap();
    let error = finish(
        &p,
        selected_request(
            &p,
            one.view().boundary(),
            two,
            first.hyphenation_continuation(),
            true,
        ),
    )
    .unwrap_err();
    hyphenation_reason(&error, HyphenationError::UncapturedIndicationFontQuery);
}

fn mixed_direction_fixture() -> Vec<u8> {
    // Retained licensed Ahem outlines only. The optional XX substitution cannot
    // match any source/indication here: no glyph58,58 sequence is emitted.
    crate::shaping_test_fonts::range_ligature_ahem(
        &[58, 58],
        &[
            (0x41, 35),
            (0x42, 36),
            (0x43, 37),
            (0x44, 38),
            (0x45, 39),
            (0x301, 100),
            (0x627, 58),
            (0x628, 35),
        ],
    )
}
#[test]
fn local_rtl_reversal_cannot_split_the_actual_cross_seam_grapheme_unit() {
    let bytes = mixed_direction_fixture();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    for (cp, id) in [
        (0x41_u32, 35_u32),
        (0x42, 36),
        (0x43, 37),
        (0x44, 38),
        (0x45, 39),
        (0x301, 100),
        (0x627, 58),
        (0x628, 35),
    ] {
        assert_eq!(font.charmap().map(cp).unwrap().to_u32(), id);
    }
    for (text, conflict) in [("\u{627}", false), ("AB", false), ("\u{627}\u{628}", true)] {
        let p = capture(
            Source::new("AB\u{ad}\u{301}CDE"),
            own_style(text, HyphenationPlacement::LogicalAfter),
            &bytes,
        );
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [65, 66, 173, 0x301, 67, 68, 69]
        );
        let receiver = p
            .unit_views()
            .find(|v| v.source_range() == Range::new(3, 4))
            .expect("original leading NSM is its genuine unit after Control SHY");
        assert_eq!(receiver.level().get(), 0);
        let offer = cut(&p, 3, counts(2, 3));
        let first = finish(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
        )
        .unwrap();
        assert_eq!(first.feedback().advance, 20.0);
        let result = finish(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                end(&p),
                first.hyphenation_continuation(),
                false,
            ),
        );
        if conflict {
            // Local AL,AL -> R,R order is BEH,ALEF; original acute keeps L0.
            // Combined logical EGC [BEH,acute] would be split by ALEF in the
            // required hierarchy. A direction boundary alone is not this error.
            hyphenation_reason(
                &result.unwrap_err(),
                HyphenationError::UnsupportedIndicationComposition,
            );
        } else {
            let line = result.unwrap();
            assert_eq!(line.prepared_text(), p);
            assert_eq!(line.feedback().baseline, 8.0);
            assert!(line.run_views().all(|run| run.face().bytes() == bytes));
        }
    }
}

#[test]
fn full_receiving_level_and_real_formatting_receiver_remain_distinct_from_local_levels() {
    for (text, paragraph_direction, outer, local) in [
        ("AB\u{ad}CDE", ParagraphBaseDirection::RightToLeft, 2, 0),
        (
            "AB\u{ad}\u{200f}CDE",
            ParagraphBaseDirection::LeftToRight,
            1,
            2,
        ),
    ] {
        let style = own_style("A", HyphenationPlacement::LogicalAfter);
        let source = Source::new(text);
        let len = source.text().len();
        let p = capture_input(
            ParagraphInput {
                source: source.clone(),
                base_style: style,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: paragraph_direction,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![paragraph::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, len),
                    aggregate: Range::new(0, len),
                    kind: paragraph::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            AHEM,
        );
        let offer = cut(&p, 3, counts(2, 3));
        let mut first = selected_request(&p, start(&p), offer.clone(), None, true);
        first.context.base_direction = paragraph_direction;
        first.context.line_box_direction = match paragraph_direction {
            crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
            crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
        };
        let first = finish(&p, first).unwrap();
        let mut next = selected_request(
            &p,
            offer.view().boundary(),
            end(&p),
            first.hyphenation_continuation(),
            false,
        );
        next.context.base_direction = paragraph_direction;
        next.context.line_box_direction = match paragraph_direction {
            crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
            crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
        };
        let line = finish(&p, next).unwrap();
        assert_eq!(line.feedback().advance, 40.0);
        let selected = &line.inner.selected;
        let domain = &selected.presentation.domains[0];
        assert_eq!((domain.outer, domain.base), (outer, outer & 1));
        let local_unit = &selected.presentation.units[domain.units.start];
        assert_eq!(
            selected.presentation.analysis.levels[local_unit.atoms.start],
            local
        );
        let super::super::prepared::presentation::Attachment::After {
            participant,
            atom: Some(atom),
        } = domain.attachment
        else {
            panic!("authentic original text receiver required")
        };
        assert_eq!(
            selected.prepared.analyzed.projection.atoms[atom]
                .source
                .start,
            3
        );
        assert!(matches!(
            selected.prepared.tables.participants[participant],
            super::super::prepared::ParticipantData::Text(_)
        ));
        assert_eq!(
            selected.prepared.analyzed.projection.atoms[atom].code_point,
            if outer == 1 { 0x200f } else { 67 }
        );
        // The local Latin scalar has its own 0/2 implicit level, never outer+local.
        assert_eq!(local_unit.level, local);
        assert!(
            selected
                .presentation
                .units
                .iter()
                .filter(|u| u.code_point.is_some_and(|cp| matches!(cp, 67..=69)))
                .all(|u| u.level == if outer == 1 { 0 } else { 2 })
        );
    }
}

#[test]
fn local_embedding_and_trailing_l1_do_not_escape_into_original_source() {
    let bytes = mixed_direction_fixture();
    let p = capture(
        Source::new("AB\u{ad}CDE"),
        own_style("\u{202b}\u{628} ", HyphenationPlacement::LogicalBefore),
        &bytes,
    );
    let offer = cut(&p, 3, counts(2, 3));
    let first = finish(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
    )
    .unwrap();
    assert!(first.hyphenation_continuation().is_none());
    let selected = &first.inner.selected;
    let domain = &selected.presentation.domains[0];
    assert_eq!((domain.outer, domain.base), (0, 0));
    assert_eq!(
        selected.presentation.units[domain.units.clone()]
            .iter()
            .map(|u| u.level)
            .collect::<Vec<_>>(),
        [0, 1, 0]
    );
    assert!(
        selected
            .presentation
            .units
            .iter()
            .filter(|u| matches!(u.code_point, Some(65 | 66)))
            .all(|u| u.level == 0)
    );
    let next = finish(
        &p,
        selected_request(&p, offer.view().boundary(), end(&p), None, false),
    )
    .unwrap();
    assert_eq!(next.feedback().advance, 30.0);
    assert!(next.inner.selected.presentation.domains.is_empty());
    assert!(next.inner.data.font_runs.iter().all(|r| r.level == 0));
}

#[test]
fn source_less_control_retains_its_actual_size_and_empty_after_has_no_pending_emission() {
    for text in ["A", ""] {
        let source = Source::new("ABCDE");
        let mut own = own_style(text, HyphenationPlacement::LogicalAfter);
        own.size = 20.0;
        own.line_height = LineHeight::Absolute(20.0);
        let p = capture_input(
            ParagraphInput {
                source: source.clone(),
                base_style: own_style("X", HyphenationPlacement::LogicalBefore),
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![paragraph::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, 5),
                    aggregate: Range::new(0, 5),
                    kind: paragraph::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                controls: vec![paragraph::ParagraphControl {
                    at: 2,
                    kind: paragraph::ParagraphControlKind::DiscretionaryBreak {
                        style: Box::new(own),
                    },
                }],
                anchor_order: vec![paragraph::ParagraphAnchor {
                    item: paragraph::ParagraphAnchorItem::Control(0),
                    affinity: crate::Affinity::After,
                }],
            },
            AHEM,
        );
        let offer = cut(&p, 2, counts(2, 3));
        let first = finish(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
        )
        .unwrap();
        assert_eq!(first.feedback().advance, 20.0);
        assert_eq!(first.hyphenation_continuation().is_some(), !text.is_empty());
        let second = finish(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                end(&p),
                first.hyphenation_continuation(),
                false,
            ),
        )
        .unwrap();
        assert_eq!(
            second.feedback().advance,
            if text.is_empty() { 30.0 } else { 50.0 }
        );
        let sizes = second
            .run_views()
            .map(|r| r.effective_instance().unwrap().font_size())
            .collect::<Vec<_>>();
        assert_eq!(
            sizes,
            if text.is_empty() {
                vec![10.0]
            } else {
                vec![20.0, 10.0]
            }
        );
        let placed =
            place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
        assert_eq!(placed.source().text().code_units(), [65, 66, 67, 68, 69]);
    }
}

#[test]
fn native_control_mark_range_keeps_both_fonts_and_only_outer_placed_carets() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let p = capture(
        Source::new("\u{1}\u{301}"),
        own_style("-", HyphenationPlacement::LogicalAfter),
        &bytes,
    );
    assert_eq!(
        p.boundary_views()
            .map(|b| b.source_before().get())
            .collect::<Vec<_>>(),
        [0, 2]
    );
    let line = finish(&p, request(&p)).unwrap();
    assert_eq!(line.run_views().count(), 2);
    let mut control = false;
    let mut mark = false;
    for run in line.run_views() {
        if run.face().bytes() == bytes {
            assert_eq!(run.glyphs().map(|g| g.id()).collect::<Vec<_>>(), [100]);
            mark = true;
        } else {
            assert!(run.glyphs().all(|g| g.id() != 0));
            control = true;
        }
    }
    assert!(control && mark);
    let placed = place_lines(&p, PlacementCoverage::WholeParagraph, vec![line]).unwrap();
    let positions: Vec<_> = placed.positions().collect();
    assert_eq!(
        positions
            .iter()
            .map(|p| p.source_cursor().index())
            .collect::<Vec<_>>(),
        [0, 2]
    );
    let selection = placed
        .position_selection(&positions[0], &positions[1])
        .unwrap();
    assert_eq!(selection.source_text().code_units(), [1, 0x301]);
    assert_eq!(selection.plaintext().code_units(), [1, 0x301]);
    assert!(!selection.geometry().rects().is_empty());
}

#[test]
fn native_control_indications_use_retained_actual_symbol_fonts_on_both_placements() {
    for cp in ['\0', '\u{b}', '\u{c}', '\u{85}'] {
        for placement in [
            HyphenationPlacement::LogicalBefore,
            HyphenationPlacement::LogicalAfter,
        ] {
            let p = capture(
                Source::new("AB\u{ad}CDE"),
                own_style(&cp.to_string(), placement),
                AHEM,
            );
            let offer = cut(&p, 3, counts(2, 3));
            let first = finish(
                &p,
                selected_request(&p, start(&p), offer.clone(), None, true),
            )
            .unwrap();
            let second = finish(
                &p,
                selected_request(
                    &p,
                    offer.view().boundary(),
                    end(&p),
                    first.hyphenation_continuation(),
                    false,
                ),
            )
            .unwrap();
            let mut symbols = 0;
            for line in [&first, &second] {
                for run in line.run_views().filter(|run| run.face().bytes() != AHEM) {
                    let instance = run.effective_instance().unwrap();
                    assert_eq!(run.face().catalogue_identity(), p.font_catalogue_identity());
                    for glyph in run.glyphs() {
                        let bounds = instance
                            .glyph_outline(glyph.id(), p.font_catalogue_identity())
                            .unwrap()
                            .bounds()
                            .unwrap();
                        assert!(bounds.size.width > 0.0 && bounds.size.height > 0.0);
                        assert!(glyph.advance() > 0.0);
                        symbols += 1;
                    }
                }
            }
            assert_eq!(symbols, 1);
            assert_eq!(
                p.analyzed_source().source().text().code_units(),
                [65, 66, 173, 67, 68, 69]
            );
            let placed =
                place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, second]).unwrap();
            assert_eq!(
                placed.source().text().code_units(),
                [65, 66, 173, 67, 68, 69]
            );
        }
    }
}

#[test]
fn indication_hard_separators_refuse_while_actual_tabs_keep_the_chosen_operation() {
    use icu_properties::{CodePointMapData, props::BidiClass};
    // LINE SEPARATOR is WS in bidi data, yet an actual hard line separator.
    assert_eq!(
        CodePointMapData::<BidiClass>::new().get32(0x2028),
        BidiClass::WhiteSpace
    );
    assert!(crate::engine::analysis::is_hard_separator(0x2028));
    for cp in ['\n', '\r', '\u{2028}', '\u{2029}'] {
        for placement in [
            HyphenationPlacement::LogicalBefore,
            HyphenationPlacement::LogicalAfter,
        ] {
            let source = Source::new("AB\u{ad}CDE");
            let text = format!("A{cp}");
            let input = ParagraphInput {
                source: source.clone(),
                base_style: own_style(&text, placement),
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![paragraph::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, 6),
                    aggregate: Range::new(0, 6),
                    kind: paragraph::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            };
            let analyzed = crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(
                    input,
                    crate::AnalysisTypography::HorizontalComposition,
                )
                .unwrap(),
                crate::Wrap::Word,
            );
            let mut system = System::new(SystemOptions {
                font_catalogue: FontCatalogueMode::RegisteredOnly,
            })
            .unwrap();
            system.register_fonts(AHEM.to_vec()).unwrap();
            hyphenation_reason(
                &system
                    .prepare(&analyzed, PreparationOptions::default())
                    .unwrap_err(),
                HyphenationError::UnsupportedIndicationSeparator,
            );
        }
    }
    // The existing FontAdvance policy and its genuine U+0009 survive local S/L1.
    let p = capture(
        Source::new("AB\u{ad}CDE"),
        own_style("\t", HyphenationPlacement::LogicalAfter),
        AHEM,
    );
    let offer = cut(&p, 3, counts(2, 3));
    let first = finish(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
    )
    .unwrap();
    let mut next_request = selected_request(
        &p,
        offer.view().boundary(),
        end(&p),
        first.hyphenation_continuation(),
        false,
    );
    next_request.context.tabs = TabContext::FontAdvance;
    let next = finish(&p, next_request).unwrap();
    assert_eq!(next.feedback().advance, 40.0);
    let domain = &next.inner.selected.presentation.domains[0];
    assert_eq!(
        next.inner.selected.presentation.units[domain.units.clone()]
            .iter()
            .map(|u| u.code_point)
            .collect::<Vec<_>>(),
        [Some(9)]
    );
    assert_eq!(
        next.inner.selected.presentation.units[domain.units.clone()]
            .iter()
            .map(|u| u.line_level)
            .collect::<Vec<_>>(),
        [0]
    );
}

#[test]
fn incoming_css_tab_is_glyph_free_and_retains_its_real_source_anchor() {
    let source = Source::new("AB\u{ad}CDE");
    let style = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::spaces(2.0).unwrap(),
        ..own_style("\t", HyphenationPlacement::LogicalAfter)
    };
    let input = ParagraphInput {
        source: source.clone(),
        base_style: style.clone(),
        tab_reference: Some(
            crate::ParagraphTabReference::new(style, ParagraphBaseDirection::LeftToRight).unwrap(),
        ),
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, 6),
            aggregate: Range::new(0, 6),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let p = system
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops).unwrap(),
        )
        .unwrap();
    let offer = cut(&p, 3, counts(2, 3));
    let first = finish(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
    )
    .unwrap();
    let mut r = selected_request(
        &p,
        offer.view().boundary(),
        end(&p),
        first.hyphenation_continuation(),
        false,
    );
    r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    let next = finish(&p, r).unwrap();
    assert_eq!(next.feedback().advance, 50.0);
    let glyphs: Vec<_> = next
        .run_views()
        .flat_map(|r| {
            r.glyphs()
                .map(|g| (g.id(), r.origin().x + g.pen(), g.advance(), g.range()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        glyphs,
        [
            (37, 20.0, 10.0, Range::new(3, 4)),
            (38, 30.0, 10.0, Range::new(4, 5)),
            (39, 40.0, 10.0, Range::new(5, 6))
        ]
    );
    let alternative = alternative(&offer);
    let placed = place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, next]).unwrap();
    let origin = p.origin_views().next().unwrap().handle();
    let fragments = placed
        .source_range_fragments(&origin, Range::new(2, 3))
        .unwrap();
    let units: Vec<_> = fragments.units().collect();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].line_index(), 1);
    assert_eq!(units[0].alternatives().collect::<Vec<_>>(), [alternative]);
    assert_eq!(
        units[0]
            .defining_sources()
            .map(|v| v.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(2, 3)]
    );
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 66, 173, 67, 68, 69]
    );
}

#[test]
fn transparent_oof_keeps_the_actual_after_prefix_and_receiving_source_custody() {
    for at in [3, 6] {
        let mut source = Source::new("AB\u{ad}CDE");
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(91223),
            crate::InlineBoxKind::OutOfFlow,
            at,
            crate::Size::new(0.0, 0.0),
        ));
        let p = capture(
            source,
            own_style("A", HyphenationPlacement::LogicalAfter),
            AHEM,
        );
        let offer = cut(&p, 3, counts(2, 3));
        let first = finish(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
        )
        .unwrap();
        let mut r = selected_request(
            &p,
            offer.view().boundary(),
            end(&p),
            first.hyphenation_continuation(),
            false,
        );
        r.atomics = r
            .participants
            .iter()
            .filter_map(|v| {
                if let PreparedParticipant::Atomic(occurrence) = v {
                    Some(AtomicAllocation {
                        occurrence: occurrence.clone(),
                        allocation: AtomicLineAllocation::OutOfFlow,
                    })
                } else {
                    None
                }
            })
            .collect();
        let line = finish(&p, r).unwrap();
        assert_eq!(line.feedback().advance, 40.0);
        assert_eq!(
            (
                line.inner.selected.presentation.domains[0].outer,
                line.inner.selected.presentation.domains[0].base
            ),
            (0, 0)
        );
        assert_eq!(
            glyphs(&line, AHEM)
                .into_iter()
                .map(|g| (g.id, g.advance, g.x, g.y))
                .collect::<Vec<_>>(),
            [
                (35, 10.0, 0.0, 0.0),
                (37, 10.0, 10.0, 0.0),
                (38, 10.0, 20.0, 0.0),
                (39, 10.0, 30.0, 0.0)
            ]
        );
        assert_eq!(line.inner.data.boxes.len(), 1);
        assert!(matches!(
            line.inner.data.boxes[0].geometry,
            crate::engine::layout::data::FinalizedBoxGeometry::OutOfFlow
        ));
        assert_eq!(p.atomic_views().next().unwrap().source_position().get(), at);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [65, 66, 173, 67, 68, 69]
        );
        let placed = place_lines(&p, PlacementCoverage::WholeParagraph, vec![first, line]).unwrap();
        assert_eq!(placed.source().boxes().len(), 1);
        let origin = p.origin_views().next().unwrap().handle();
        let fragments = placed
            .source_range_fragments(&origin, Range::new(2, 3))
            .unwrap();
        let units: Vec<_> = fragments.units().collect();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].line_index(), 1);
        assert_eq!(
            units[0].alternatives().collect::<Vec<_>>(),
            [alternative(&offer)]
        );
    }
}

#[test]
fn transparent_oof_does_not_split_the_actual_mixed_in_flow_rtl_run() {
    let mut observed = Vec::new();
    for with_oof in [false, true] {
        let mut source = Source::new("A");
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(91300),
            crate::InlineBoxKind::InFlow,
            1,
            crate::Size::new(10.0, 10.0),
        ));
        if with_oof {
            source.inline_box(crate::InlineBox::new(
                crate::Id::from_u64(91301),
                crate::InlineBoxKind::OutOfFlow,
                1,
                crate::Size::new(0.0, 0.0),
            ));
        }
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(91302),
            crate::InlineBoxKind::InFlow,
            1,
            crate::Size::new(20.0, 10.0),
        ));
        let last = if with_oof { 2 } else { 1 };
        let mut anchors = vec![paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(0),
            affinity: crate::Affinity::After,
        }];
        anchors.extend(
            (0..source.boxes().len()).map(|i| paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Atomic(i),
                affinity: crate::Affinity::After,
            }),
        );
        let input = ParagraphInput {
            source: source.clone(),
            base_style: style(),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![paragraph::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, 1),
                aggregate: Range::new(0, 1),
                kind: paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![paragraph::ParagraphControl {
                at: 1,
                kind: paragraph::ParagraphControlKind::RightToLeftEmbedding,
            }],
            boundary_policies: [0, last]
                .into_iter()
                .flat_map(|i| {
                    [
                        paragraph::ParagraphBoundaryDefinition::AtomicBefore(i),
                        paragraph::ParagraphBoundaryDefinition::AtomicAfter(i),
                    ]
                })
                .map(|definition| paragraph::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            anchor_order: anchors,
        };
        let mut system = system();
        let analyzed = crate::test_wrapping::bind_missing(
            &AnalyzedSource::from_paragraph(
                input,
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap(),
            crate::Wrap::Word,
        );
        let p = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        assert_eq!(p.inner.analyzed.analysis.levels, [0]);
        assert_eq!(p.inner.analyzed.analysis.control_levels, [0]);
        assert_eq!(
            p.inner.analyzed.analysis.box_levels,
            if with_oof { vec![1, 0, 1] } else { vec![1, 1] }
        );
        let mut r = request(&p);
        r.context.spacing = SpacingPolicy::None;
        r.controls = r
            .participants
            .iter()
            .filter_map(|v| {
                if let PreparedParticipant::Control(c) = v {
                    Some({
                        let occurrence = c.clone();
                        ControlAllocation {
                            boundary: matches!(
                                occurrence.view().kind(),
                                crate::ParagraphControlKind::InlineBoundaryStart
                                    | crate::ParagraphControlKind::InlineBoundaryEnd
                            )
                            .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                            occurrence,
                            strut: None,
                        }
                    })
                } else {
                    None
                }
            })
            .collect();
        r.atomics = r
            .participants
            .iter()
            .filter_map(|v| {
                if let PreparedParticipant::Atomic(a) = v {
                    let allocation =
                        if a.view().original_box().kind() == crate::InlineBoxKind::OutOfFlow {
                            AtomicLineAllocation::OutOfFlow
                        } else {
                            let width = a.view().original_box().size().width;
                            AtomicLineAllocation::InFlow {
                                fit: crate::AtomicFitAllocation::NativeOccupied,
                                flow: AtomicFlowAllocation {
                                    advance: width,
                                    before_baseline: 8.0,
                                    after_baseline: 2.0,
                                    occupied_start: 0.0,
                                    occupied_end: width,
                                },
                                box_geometry: AtomicBoxAllocation {
                                    border: crate::Rect::new(0.0, 0.0, width, 10.0),
                                    content: crate::Rect::new(0.0, 0.0, width, 10.0),
                                    baseline: 8.0,
                                },
                                alignment: LineBoxAlignment::Baseline,
                            }
                        };
                    Some(AtomicAllocation {
                        occurrence: a.clone(),
                        allocation,
                    })
                } else {
                    None
                }
            })
            .collect();
        let line = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(line.feedback().advance, 50.0);
        assert_eq!(line.inner.data.boxes.len(), if with_oof { 3 } else { 2 });
        let first_x = line
            .inner
            .data
            .boxes
            .iter()
            .find(|b| b.occurrence == 0)
            .unwrap()
            .slot
            .x;
        let last_x = line
            .inner
            .data
            .boxes
            .iter()
            .find(|b| b.occurrence == last)
            .unwrap()
            .slot
            .x;
        assert_eq!(p.analyzed_source().source().text().code_units(), [65]);
        assert!(
            line.run_views().all(|r| r.face().bytes()
                == include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf"))
        );
        observed.push((with_oof, first_x, last_x));
    }
    // Both genuine boxes are in the same RLE run. Omitting a nonparticipating
    // OOF anchor leaves L2 order last(width20),first(width10), after A20.
    assert_eq!(observed, [(false, 40.0, 20.0), (true, 40.0, 20.0)]);
}
