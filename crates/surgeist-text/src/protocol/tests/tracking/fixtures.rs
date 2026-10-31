//! Private actual-font and public-operation helpers for character spacing.
use super::*;
use crate::{Affinity, Cursor, Rect, Selection};

pub(super) fn select_method(prepared: &PreparedText, method: SpacingPolicy) -> SelectedLine {
    let mut value = request(prepared);
    value.context.available_inline = None;
    value.context.spacing = method;
    prepared.select(value).unwrap()
}

pub(super) fn allocations(selected: &SelectedLine, amounts: &[f64]) -> SpacingAllocations {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), amounts.len());
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip(amounts)
            .map(|(slot, &amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}

pub(super) fn abc_premises() {
    font_premises(AHEM, false);
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.charmap().map('C'), Some(GlyphId::new(37)));
    assert_eq!(
        font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(37)),
        Some(1000.0)
    );
}

/// Actual GSUB type4 fi ligation under the requested real OpenType feature.
/// The existing builder's ccmp tag is replaced in its feature record, not used
/// as evidence for optional or required feature semantics.
pub(in crate::protocol::tests) fn ligature_font(tag: [u8; 4]) -> Vec<u8> {
    assert!([*b"liga", *b"clig", *b"rlig"].contains(&tag));
    let base = crate::shaping_test_fonts::range_ligature_ahem(
        &[35, 36],
        &[(0x41, 35), (0x42, 36), (0x43, 37), (0x66, 35), (0x69, 36)],
    );
    let mut gsub = sfnt::table(&base, *b"GSUB").to_vec();
    let features = usize::from(word(&gsub, 6));
    assert_eq!(word(&gsub, features), 1);
    assert_eq!(&gsub[features + 2..features + 6], b"ccmp");
    gsub[features + 2..features + 6].copy_from_slice(&tag);
    let bytes = sfnt::replace_tables(&base, vec![(*b"GSUB", gsub)]);
    font_premises(&bytes, false);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('f'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map('i'), Some(GlyphId::new(36)));
    assert_eq!(
        font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(58)),
        Some(1000.0)
    );
    assert!(font.gsub().unwrap().lookup_list().is_ok());
    let gsub = sfnt::table(&bytes, *b"GSUB");
    let features = usize::from(word(gsub, 6));
    assert_eq!(&gsub[features + 2..features + 6], &tag);
    let feature = features + usize::from(word(gsub, features + 6));
    assert_eq!((word(gsub, feature + 2), word(gsub, feature + 4)), (1, 0));
    let list = usize::from(word(gsub, 8));
    assert_eq!(word(gsub, list), 1);
    let lookup = list + usize::from(word(gsub, list + 2));
    assert_eq!((word(gsub, lookup), word(gsub, lookup + 4)), (4, 1));
    let sub = lookup + usize::from(word(gsub, lookup + 6));
    assert_eq!((word(gsub, sub), word(gsub, sub + 4)), (1, 1));
    let coverage = sub + usize::from(word(gsub, sub + 2));
    assert_eq!(
        (
            word(gsub, coverage),
            word(gsub, coverage + 2),
            word(gsub, coverage + 4)
        ),
        (1, 1, 35)
    );
    let set = sub + usize::from(word(gsub, sub + 6));
    assert_eq!(word(gsub, set), 1);
    let ligature = set + usize::from(word(gsub, set + 2));
    assert_eq!(
        (
            word(gsub, ligature),
            word(gsub, ligature + 2),
            word(gsub, ligature + 4)
        ),
        (58, 2, 36)
    );
    bytes
}

/// Observe the adopted predecessor-owned shared cells through actual placement.
/// All callers supply BMP Latin units with size10/Absolute10 and raw Ahem8/2.
pub(super) fn assert_cells(
    prepared: &PreparedText,
    line: FinalizedLine,
    source: &Source,
    edges: &[f64],
) {
    assert_eq!(edges.len(), source.text().len() + 1);
    let total = *edges.last().unwrap();
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 70.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), source);
    assert_eq!(layout.bounds(), Rect::new(20.0, 62.0, total, 10.0));
    for (index, &edge) in edges.iter().enumerate() {
        let affinity = if index + 1 == edges.len() {
            Affinity::Before
        } else {
            Affinity::After
        };
        assert_eq!(
            layout.cursor(Cursor::new(index, affinity)).unwrap().rect(),
            Rect::new(20.0 + edge, 62.0, 1.0, 10.0)
        );
    }
    for (index, pair) in edges.windows(2).enumerate() {
        let selection = layout
            .selection(Selection::new(
                Cursor::new(index, Affinity::After),
                Cursor::new(index + 1, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(selection.rects().len(), 1);
        assert_eq!(
            selection.rects()[0].rect(),
            Rect::new(20.0 + pair[0], 62.0, pair[1] - pair[0], 10.0)
        );
    }
}
