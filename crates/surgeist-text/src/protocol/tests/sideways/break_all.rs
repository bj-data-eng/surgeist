//! CSS Text3 BreakAll applies to Hebrew typographic letters, not only AL/NU/SA.
use super::*;
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, LineBreak as UnicodeLineBreak},
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

fn prepared_pair(system: &mut System, word_break: crate::WordBreak) -> PreparedText {
    let prepared = capture(
        system,
        "\u{5d0}\u{5d0}",
        Style {
            word_break,
            ..own_style()
        },
        true,
        CW,
    );
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [0x5d0, 0x5d0]
    );
    assert_eq!(
        prepared.analyzed_source().direction(),
        crate::Direction::RightToLeft
    );
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| (unit.source_range(), unit.level().get(), unit.advance()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 1, 10.0), (Range::new(1, 2), 1, 10.0),]
    );
    prepared
}

#[test]
fn break_all_offers_real_hebrew_letter_cut_and_finalizes_two_ordinary_rtl_lines() {
    // Pinned CSS Text3 defines a typographic letter by Letter/Number general
    // category, and BreakAll treats those units as ID for line breaking. The
    // actual installed data identify Alef as Lo/HL, beyond ICU's AL/NU/SA set.
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get('\u{5d0}'),
        GeneralCategory::OtherLetter
    );
    assert_eq!(
        CodePointMapData::<UnicodeLineBreak>::new().get('\u{5d0}'),
        UnicodeLineBreak::HebrewLetter
    );
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.charmap().map('\u{5d0}'), Some(GlyphId::new(58)));
    assert!(font.gsub().is_ok());
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for glyph in [58, 274] {
        assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(1000.0));
    }
    let mut system = system_with(bytes.clone());
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), bytes);
    assert_eq!(face.index(), 0);
    assert_eq!(
        face.unsynthesized_instance(10.0, &[]).unwrap().axis_count(),
        0
    );
    for rule in [crate::WordBreak::Normal, crate::WordBreak::KeepAll] {
        let control = prepared_pair(&mut system, rule);
        assert_eq!(
            control
                .opportunity_views()
                .filter(|offer| offer.kind() == PreparedBreakKind::Allowed)
                .count(),
            0
        );
        assert_eq!(whole(&control).feedback().advance, 20.0);
    }
    let prepared = prepared_pair(&mut system, crate::WordBreak::BreakAll);
    let cut = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().source_after().get() == 1
        })
        .expect("BreakAll must offer an ordinary cut between actual Hebrew letter units")
        .handle();
    let next = cut.view().boundary();
    let mut prefix = selected_request(&prepared, start(&prepared), cut.clone());
    prefix.context.available_inline = Some(10.0);
    let first = prepared
        .select(prefix)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let end = terminal(&prepared);
    let mut suffix = selected_request(&prepared, next, end.clone());
    suffix.context.available_inline = Some(10.0);
    let second = prepared
        .select(suffix)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    for (line, range) in [(&first, Range::new(0, 1)), (&second, Range::new(1, 2))] {
        assert_eq!(line.prepared_text(), prepared);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (10.0, 8.0, 10.0)
        );
        assert_eq!(line.measurements().fit().advance(), 10.0);
        let selection = line.selected_line();
        assert_eq!(selection.request().context.available_inline, Some(10.0));
        assert_eq!(
            selection.request().context.base_direction,
            ParagraphBaseDirection::RightToLeft
        );
        assert_eq!(selection.request().participants.len(), 1);
        assert_eq!(
            line.run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [(range, 274, 10.0)]
        );
    }
    assert_eq!(first.selected_line().request().context.end, LineEnd::Soft);
    assert_eq!(
        first.selected_line().request().termination,
        PreparedTermination::Ordinary(cut)
    );
    assert_eq!(
        second.selected_line().request().context.start,
        LineStart::Continuation
    );
    assert_eq!(
        second.selected_line().request().context.end,
        LineEnd::Paragraph
    );
    assert_eq!(
        second.selected_line().request().termination,
        PreparedTermination::Ordinary(end)
    );
    let layout = placed(&prepared, vec![(first, 20.0, 100.0), (second, 20.0, 80.0)]);
    assert_eq!(
        layout.analyzed_source().identity(),
        prepared.analyzed_source().identity()
    );
    assert_eq!(layout.source(), prepared.analyzed_source().source());
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| (line.range(), line.bounds()))
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), Rect::new(98.0, 20.0, 10.0, 10.0)),
            (Range::new(1, 2), Rect::new(78.0, 20.0, 10.0, 10.0)),
        ]
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(98.0, 30.0, 10.0, 1.0)
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(1, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(78.0, 30.0, 10.0, 1.0)
    );
    for run in layout.run_views() {
        assert_eq!(run.glyph_basis(), crate::GlyphBasis::Clockwise);
        assert_eq!(run.face().bytes(), bytes);
    }
}
