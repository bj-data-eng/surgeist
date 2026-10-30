//! Exact character allocations remain semantic custody when geometry agrees.
use super::*;
use crate::{OverflowWrap, Point, Rect};

struct Prefix {
    prepared: PreparedText,
    selected: SelectedLine,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
    slot: SpacingSlot,
}

fn prefix() -> Prefix {
    font_premises(AHEM, false);
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.charmap().map(' '), Some(GlyphId::new(3)));
    assert_eq!(
        font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(3)),
        Some(1000.0)
    );
    let source = Source::identified("A A", Id::from_u64(1566070), 7);
    let prepared = prepare(
        AHEM,
        &source,
        Style {
            white_space: WhiteSpace::BreakSpaces,
            overflow_wrap: OverflowWrap::Anywhere,
            ..own(0.0)
        },
        &UNITS,
    );
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
    value.context.spacing = SpacingPolicy::InterCharacter;
    value.context.alignment = crate::Alignment::Start;
    let selected = prepared.select(value).unwrap();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let slot = slots[0].clone();
    boundary_custody(&selected, 0, 0.0);
    assert!(matches!(slot.target(), SpacingSlotTarget::Boundary { .. }));
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.feedback().advance, 20.0);
    assert_eq!(paint_offsets(&natural), [(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(
        raw(&natural, AHEM),
        [
            (UNITS[0], 35, 0.0, 0.0, 10.0),
            (UNITS[1], 3, 0.0, 0.0, 10.0),
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

fn amounts() -> (f64, f64) {
    let one = 1.0_f64;
    let next = f64::from_bits(one.to_bits() + 1);
    assert!(next > one);
    assert_ne!(one.to_bits(), next.to_bits());
    // ULP(1)=2^-52; ULP(10) and ULP(20) are larger. The two accepted
    // complete boundary amounts therefore produce the same paint/cell extent.
    assert_eq!((10.0 + one, 10.0 + next), (11.0, 11.0));
    assert_eq!((20.0 + one, 20.0 + next), (21.0, 21.0));
    (one, next)
}

fn allocated(prefix: &Prefix, amount: f64) -> FinalizedLine {
    let natural = prefix
        .selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let line = natural
        .with_spacing(SpacingAllocations::Complete(vec![SpacingAllocation {
            slot: prefix.slot.clone(),
            amount,
        }]))
        .unwrap();
    assert_eq!(line.prepared_text(), prefix.prepared);
    assert_eq!(line.selected_line(), prefix.selected);
    assert_eq!(line.feedback().advance, 21.0);
    assert_eq!(line.measurements().alignment().advance(), 21.0);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (11.0, 0.0)]);
    assert_eq!(raw(&line, AHEM), raw(&natural, AHEM));
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].slot, prefix.slot);
    assert_eq!(rows[0].allocation.to_bits(), amount.to_bits());
    assert_eq!(rows[0].authored_word_spacing, 0.0);
    assert_eq!(rows[0].authored_letter_spacing, 0.0);
    assert_eq!(
        rows[0].geometry,
        SpacingSlotGeometry::Boundary(BoundaryGeometry::Independent)
    );
    assert_eq!(rows[0].stretchability, SpacingStretchability::Direct);
    assert_eq!(natural.feedback().advance, 20.0);
    line
}

fn placed(prefix: &Prefix, line: FinalizedLine) -> crate::Layout {
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
    assert_eq!(layout.source().revision(), 7);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 2));
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(100.25, 192.5, 40.0, 10.0)
    );
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        Point::new(100.25, 200.5)
    );
    let origin = prefix.prepared.origin_views().next().unwrap().handle();
    let ranges = layout
        .source_range_fragments(&origin, Range::new(0, 2))
        .unwrap();
    assert_eq!(
        ranges
            .units()
            .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            Rect::new(100.25, 192.5, 11.0, 10.0),
            Rect::new(111.25, 192.5, 10.0, 10.0),
        ]
    );
    layout
}

#[test]
fn rounded_equal_character_replacements_keep_exact_placement_key_custody() {
    let prefix = prefix();
    let (one, next) = amounts();
    let first = allocated(&prefix, one);
    let repeated = allocated(&prefix, one);
    let different = allocated(&prefix, next);
    assert_ne!(first, repeated);
    assert_eq!(first.feedback(), different.feedback());
    let first = placed(&prefix, first);
    let repeated = placed(&prefix, repeated);
    let different = placed(&prefix, different);
    assert_eq!(first.lines(), different.lines());
    assert_eq!(first.bounds(), different.bounds());
    assert_eq!(first.key(), repeated.key());
    assert_ne!(first.key(), different.key());
}

#[test]
fn rounded_equal_character_replacements_keep_exact_framed_conditional_identity() {
    let prefix = prefix();
    let (one, next) = amounts();
    let first = allocated(&prefix, one);
    let repeated = allocated(&prefix, one);
    let different = allocated(&prefix, next);
    assert_eq!(first.feedback(), different.feedback());
    let query = |frame: &FinalizedLine| {
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
                Some(frame),
            )
            .unwrap();
        search.begin(&[]).unwrap();
        search
    };
    let first = query(&first);
    let repeated = query(&repeated);
    let different = query(&different);
    assert_eq!(first, repeated);
    assert_ne!(first, different);
}
