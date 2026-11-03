//! Exact supplied spacing remains semantic custody when binary64 geometry agrees.
use super::*;
use crate::{LineHeight, OverflowWrap, Point, Range, Rect};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Prefix {
    prepared: PreparedText,
    selected: SelectedLine,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
    slot: SpacingSlot,
}

fn prefix() -> Prefix {
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map(' '), Some(GlyphId::new(3)));
    let raw = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for glyph in [35, 3] {
        assert_eq!(raw.advance_width(GlyphId::new(glyph)), Some(1000.0));
    }
    let mut system = system();
    let source = Source::new("A A");
    let input = ParagraphInput {
        source: source.clone(),
        base_style: Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            line_height: LineHeight::Absolute(10.0),
            white_space: WhiteSpace::BreakSpaces,
            overflow_wrap: OverflowWrap::Anywhere,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            ..Style::default()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(0, 3),
            aggregate: Range::new(0, 3),
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
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.baseline_reference(), None);
    let start = prepared.boundary_views().next().unwrap().handle();
    let first = prepared.opportunity_views().next().unwrap();
    assert_eq!(first.kind(), PreparedBreakKind::Allowed);
    assert_eq!(first.boundary().view().source_after().get(), 2);
    let offer = first.handle();
    let end = offer.view().boundary();
    let mut value = request(&prepared);
    value.start = start.clone();
    value.end = end.clone();
    value.termination = PreparedTermination::Ordinary(offer.clone());
    value.participants = prepared
        .participants()
        .take(end.view().participant_offset())
        .collect();
    value.context.start = LineStart::Paragraph;
    value.context.end = LineEnd::Soft;
    value.context.available_inline = Some(40.0);
    value.context.spacing = SpacingPolicy::Spaces;
    value.context.alignment = crate::Alignment::Start;
    let selected = prepared.select(value).unwrap();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let slot = slots[0].clone();
    assert_eq!(
        slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
        [crate::SpacingSlotOwner::Source {
            origin: prepared.origin_views().next().unwrap().handle(),
            local: Range::new(1, 2),
            aggregate: Range::new(1, 2),
        }]
    );
    assert_eq!(slot.selected_line(), selected);
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.feedback().advance, 20.0);
    assert_eq!(natural.spacing_slots().next().unwrap().advance, 10.0);
    assert!(natural.has_in_flow_content());
    assert_eq!(natural.prepared_text(), prepared);
    assert_eq!(
        raw_glyphs(&natural),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 3, 10.0, 0.0, 0.0, 10.0)
        ]
    );
    Prefix {
        prepared,
        selected,
        start,
        offer,
        slot,
    }
}

fn raw_glyphs(line: &FinalizedLine) -> Vec<(Range, u32, f64, f32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        g.id(),
                        run.origin().x + g.pen(),
                        g.x(),
                        g.y(),
                        g.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn allocated(prefix: &Prefix, amount: f64) -> FinalizedLine {
    let line = prefix
        .selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::Complete(vec![SpacingAllocation {
                slot: prefix.slot.clone(),
                amount,
            }]),
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.prepared_text(), prefix.prepared);
    assert_eq!(line.selected_line(), prefix.selected);
    assert_eq!(line.selected_line().request(), prefix.selected.request());
    assert_eq!(line.feedback().advance, 21.0);
    assert!(line.has_in_flow_content());
    let feedback = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(feedback.len(), 1);
    assert_eq!(feedback[0].slot, prefix.slot);
    assert_eq!(feedback[0].advance, 11.0);
    assert_eq!(
        feedback[0].disposition,
        SpacingSlotDisposition::Retained { trailing: true }
    );
    assert_eq!(
        raw_glyphs(&line),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 3, 10.0, 0.0, 0.0, 10.0)
        ]
    );
    // This private premise only confirms that the real executed contribution
    // survives. The regression oracles below use public key/query equality.
    let space = line
        .inner
        .data
        .clusters
        .iter()
        .find(|row| row.source == Range::new(1, 2))
        .unwrap();
    assert_eq!(space.logical_adjustments[2].to_bits(), amount.to_bits());
    line
}

fn amounts() -> (f64, f64) {
    let one = 1.0_f64;
    let next = f64::from_bits(one.to_bits() + 1);
    assert_ne!(one.to_bits(), next.to_bits());
    assert!(next > one);
    // ULP(1)=2^-52, whereas ULP(11)=2^-49. Both exact inputs round
    // to SPACE11 and total21; equal geometry cannot identify their custody.
    assert_eq!((10.0 + one, 10.0 + next), (11.0, 11.0));
    assert_eq!((20.0 + one, 20.0 + next), (21.0, 21.0));
    (one, next)
}

fn placed_prefix(prefix: &Prefix, line: FinalizedLine) -> crate::Layout {
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prefix.prepared.clone(),
        coverage: PlacementCoverage::Range {
            start: prefix.start.clone(),
            end: prefix.offer.view().boundary(),
        },
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 100.25,
                baseline: 200.5,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), prefix.prepared.analyzed_source().source());
    assert_eq!(
        layout.analyzed_source().identity(),
        prefix.prepared.analyzed_source().identity()
    );
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 2));
    // The finite line metric frame retains the requested40; its actual flow
    // advance21 and the source Range pieces10/11 remain separate observations.
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(100.25, 192.5, 40.0, 10.0)
    );
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        Point::new(100.25, 200.5)
    );
    let origin = prefix.prepared.origin_views().next().unwrap().handle();
    let range = layout
        .source_range_fragments(&origin, Range::new(0, 2))
        .unwrap();
    assert_eq!(range.units().count(), 2);
    assert_eq!(
        range
            .units()
            .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            Rect::new(100.25, 192.5, 10.0, 10.0),
            Rect::new(110.25, 192.5, 11.0, 10.0)
        ]
    );
    layout
}

#[test]
fn formatted_key_retains_exact_spacing_amounts_when_geometry_rounds_equal() {
    let prefix = prefix();
    let (one, next) = amounts();
    let first = allocated(&prefix, one);
    let repeated = allocated(&prefix, one);
    let different = allocated(&prefix, next);
    assert_ne!(
        first, repeated,
        "independent finalization has its own receipt identity"
    );
    assert_eq!(first.feedback(), different.feedback());
    let first = placed_prefix(&prefix, first);
    let repeated = placed_prefix(&prefix, repeated);
    let different = placed_prefix(&prefix, different);
    assert_eq!(first.lines(), different.lines());
    assert_eq!(first.bounds(), different.bounds());
    assert_eq!(
        first.key(),
        repeated.key(),
        "equal replay values must have equal keys despite different receipts"
    );
    // Complete allocation/replay custody is observable even when the final
    // slot and line advances rounded to the same binary64 geometry.
    assert_ne!(
        first.key(),
        different.key(),
        "distinct accepted spacing amounts must remain distinct placement keys"
    );
}

#[test]
fn framed_conditional_search_retains_exact_spacing_amounts_when_geometry_rounds_equal() {
    let prefix = prefix();
    let (one, next) = amounts();
    let first = allocated(&prefix, one);
    let repeated = allocated(&prefix, one);
    let different = allocated(&prefix, next);
    assert_ne!(first, repeated);
    assert_eq!(first.feedback(), different.feedback());
    let query = |frame: &FinalizedLine| {
        // The frame is a real finite ordinary Soft selection from the same
        // preparation/start, not a fabricated consumer policy disposition.
        let search = prefix
            .prepared
            .source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &prefix.start,
                None,
                &prefix.offer,
                ConditionalPurpose::Overflow,
                &[],
                crate::ConditionalLineContext::Finalized(frame),
            )
            .unwrap();
        search.begin(&[]).unwrap();
        search
    };
    let first = query(&first);
    let repeated = query(&repeated);
    let different = query(&different);
    assert_eq!(
        first, repeated,
        "equal allocation replay must preserve semantic search identity"
    );
    assert_ne!(
        first, different,
        "a framed search must retain complete exact allocation custody, not only its rounded output"
    );
}
