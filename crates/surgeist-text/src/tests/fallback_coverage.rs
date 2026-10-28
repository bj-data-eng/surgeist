//! Ordered real-font coverage is independent of noncontributing format scalars.

use super::*;
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GraphemeClusterBreak, Script},
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
    string::StringId,
};

const TAG_SPACE: u32 = 0xe0020;

fn family(bytes: &[u8]) -> String {
    skrifa::FontRef::new(bytes)
        .unwrap()
        .localized_strings(StringId::FAMILY_NAME)
        .english_or_first()
        .unwrap()
        .to_string()
}

fn fonts() -> [Vec<u8>; 2] {
    let primary = crate::shaping_test_fonts::kana_ahem();
    let secondary = crate::shaping_test_fonts::mark_ahem();
    let font = skrifa::FontRef::new(&secondary).unwrap();
    let mut name = font
        .table_data(skrifa::Tag::new(b"name"))
        .unwrap()
        .as_bytes()
        .to_vec();
    // Same-length actual name-table replacement, as in the retained owned
    // hyphenation fixtures. This creates a real Bhem face, not a family alias.
    for at in 0..name.len().saturating_sub(7) {
        if &name[at..at + 8] == b"\0A\0h\0e\0m" {
            name[at + 1] = b'B';
        }
    }
    let secondary = crate::test_fonts::replace_tables(&secondary, vec![(*b"name", name)]);
    for (bytes, expected_family) in [(&primary, "Ahem"), (&secondary, "Bhem")] {
        let face = skrifa::FontRef::new(bytes).unwrap();
        assert_eq!(family(bytes), expected_family);
        assert_eq!(face.head().unwrap().units_per_em(), 1000);
        let hhea = face.hhea().unwrap();
        assert_eq!(
            (
                hhea.ascender().to_i16(),
                hhea.descender().to_i16(),
                hhea.line_gap().to_i16(),
            ),
            (800, -200, 0)
        );
        assert_eq!(face.charmap().map(0x41_u32).unwrap().to_u32(), 35);
        assert!(face.charmap().map(0x44_u32).is_none());
        assert!(face.charmap().map(TAG_SPACE).is_none());
        assert_eq!(
            face.glyph_metrics(FontSize::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(0)),
            Some(1000.0)
        );
    }
    let primary_face = skrifa::FontRef::new(&primary).unwrap();
    assert!(primary_face.charmap().map(0x58_u32).is_none());
    let secondary_face = skrifa::FontRef::new(&secondary).unwrap();
    assert_eq!(secondary_face.charmap().map(0x58_u32).unwrap().to_u32(), 58);
    assert_eq!(
        secondary_face
            .glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(58)),
        Some(1000.0)
    );
    [primary, secondary]
}

fn style() -> Style {
    let value = Style {
        font: Font::new().family("Ahem").family("Bhem"),
        size: 10.0,
        line_height: LineHeight::MetricsRelative(1.0),
        ..Style::default()
    };
    assert_eq!(value.font.family, ["Ahem", "Bhem"]);
    value
}

fn options() -> Options {
    Options {
        quantize: false,
        ..Options::default()
    }
}

fn registered(bytes: &[Vec<u8>; 2]) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    for face in bytes {
        system.register_fonts(face.clone()).unwrap();
    }
    system
}

fn analysis(text: &str, id: u64) -> AnalyzedSource {
    // The tag is Format/Common/Extend, not a Cc control or an authored native
    // ParagraphControl. Its genuine GB9 continuation is the reachability proof.
    assert_eq!(
        CodePointMapData::<GraphemeClusterBreak>::new().get32(TAG_SPACE),
        GraphemeClusterBreak::Extend
    );
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get32(TAG_SPACE),
        GeneralCategory::Format
    );
    assert_eq!(
        CodePointMapData::<Script>::new().get32(TAG_SPACE),
        Script::Common
    );
    assert_eq!(CodePointMapData::<Script>::new().get32(0x58), Script::Latin);
    let source = Source::identified(text, Id::from_u64(id), 17);
    let analyzed = AnalyzedSource::new(
        source.clone(),
        style(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.direction(), Direction::LeftToRight);
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.source().id(), Some(Id::from_u64(id)));
    assert_eq!(analyzed.source().revision(), 17);
    assert_eq!(
        analyzed.source().text().code_units(),
        text.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [Range::new(0, source.text().len())]
    );
    analyzed
}

fn assert_source(layout: &Layout, analyzed: &AnalyzedSource) {
    assert_eq!(layout.source(), analyzed.source());
    assert_eq!(
        layout.analyzed_source().graphemes().collect::<Vec<_>>(),
        analyzed.graphemes().collect::<Vec<_>>()
    );
}

fn assert_face(layout: &Layout, bytes: &[u8], expected_family: &str) {
    assert!(
        !layout.glyph_runs().is_empty(),
        "actual metric or paint run must exist"
    );
    for run in layout.glyph_runs() {
        assert_eq!(
            family(run.font().bytes()),
            expected_family,
            "actual selected resource family"
        );
        assert!(
            run.font().bytes() == bytes,
            "actual selected resource bytes"
        );
        assert_eq!(run.font().index(), 0);
        assert_eq!(
            run.font().catalogue_identity(),
            layout.key().font_catalogue()
        );
    }
    for run in layout.run_views() {
        assert!(run.face().bytes() == bytes);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
    }
}

fn assert_visible(
    layout: &Layout,
    analyzed: &AnalyzedSource,
    bytes: &[u8],
    expected_family: &str,
    glyph: u32,
) {
    assert_source(layout, analyzed);
    // Width alone would miss this defect: both real faces advance10. Check
    // the selected resource before inspecting its actual provider glyph.
    assert_face(layout, bytes, expected_family);
    let runs = layout.glyph_runs();
    let glyphs = runs
        .iter()
        .flat_map(|run| run.glyphs())
        .map(|g| (g.id(), g.advance(), g.range()))
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [(glyph, 10.0, Range::new(0, analyzed.source().text().len()))]
    );
    assert_eq!(layout.metrics().full_width(), 10.0);
}

#[test]
fn contributing_x_with_tag_space_uses_the_later_face_that_covers_x() {
    let bytes = fonts();
    let analyzed = analysis("X\u{e0020}", 1_447_001);
    assert_eq!(
        analyzed.source().text().code_units(),
        [0x58, 0xdb40, 0xdc20]
    );
    let mut system = registered(&bytes);
    let old = system.layout_analyzed(&analyzed, options()).unwrap();
    assert_visible(&old, &analyzed, &bytes[1], "Bhem", 58);
    // These are prospective until the selected-resource assertion succeeds.
    let stats = system.stats();
    let warm = system.layout_analyzed(&analyzed, options()).unwrap();
    assert_eq!(system.stats().layout_hits(), stats.layout_hits() + 1);
    assert_eq!(warm.key(), old.key());
    assert_visible(&warm, &analyzed, &bytes[1], "Bhem", 58);
    let mut cold_system = registered(&bytes);
    let cold = cold_system.layout_analyzed(&analyzed, options()).unwrap();
    assert_visible(&cold, &analyzed, &bytes[1], "Bhem", 58);
    let old_publication = old.key().font_catalogue().clone();
    system.refresh_fonts().unwrap();
    let refreshed = system.layout_analyzed(&analyzed, options()).unwrap();
    assert_ne!(refreshed.key().font_catalogue(), &old_publication);
    assert_visible(&refreshed, &analyzed, &bytes[1], "Bhem", 58);
    drop(system);
    drop(cold_system);
    drop(bytes);
    let expected = fonts();
    assert_eq!(old.key().font_catalogue(), &old_publication);
    assert_visible(&old, &analyzed, &expected[1], "Bhem", 58);
    assert_visible(&warm, &analyzed, &expected[1], "Bhem", 58);
    assert_visible(&refreshed, &analyzed, &expected[1], "Bhem", 58);
}

#[test]
fn ordinary_x_reaches_the_covering_later_registered_face() {
    let bytes = fonts();
    let analyzed = analysis("X", 1_447_002);
    let mut system = registered(&bytes);
    let old = system
        .layout(analyzed.source().clone(), style(), options())
        .unwrap();
    assert_visible(&old, &analyzed, &bytes[1], "Bhem", 58);
    let before = system.stats();
    let warm = system
        .layout(analyzed.source().clone(), style(), options())
        .unwrap();
    assert_eq!(system.stats().layout_hits(), before.layout_hits() + 1);
    assert_eq!(warm.key(), old.key());
    system.refresh_fonts().unwrap();
    let refreshed = system
        .layout(analyzed.source().clone(), style(), options())
        .unwrap();
    assert_ne!(refreshed.key().font_catalogue(), old.key().font_catalogue());
    assert_visible(&refreshed, &analyzed, &bytes[1], "Bhem", 58);
    drop(system);
    drop(bytes);
    assert_visible(&old, &analyzed, &fonts()[1], "Bhem", 58);
}

#[test]
fn contributing_a_with_tag_space_keeps_the_first_complete_face() {
    let bytes = fonts();
    let analyzed = analysis("A\u{e0020}", 1_447_003);
    let mut system = registered(&bytes);
    let layout = system.layout_analyzed(&analyzed, options()).unwrap();
    assert_visible(&layout, &analyzed, &bytes[0], "Ahem", 35);
}

#[test]
fn genuinely_missing_d_with_tag_space_keeps_first_face_notdef_and_source() {
    let bytes = fonts();
    let analyzed = analysis("D\u{e0020}", 1_447_004);
    let mut system = registered(&bytes);
    let layout = system.layout_analyzed(&analyzed, options()).unwrap();
    assert_visible(&layout, &analyzed, &bytes[0], "Ahem", 0);
}

#[test]
fn tag_space_alone_retains_a_real_zero_glyph_font_metric_row() {
    let bytes = fonts();
    let analyzed = analysis("\u{e0020}", 1_447_005);
    assert_eq!(analyzed.source().text().code_units(), [0xdb40, 0xdc20]);
    let mut system = registered(&bytes);
    let layout = system.layout_analyzed(&analyzed, options()).unwrap();
    assert_source(&layout, &analyzed);
    assert_face(&layout, &bytes[0], "Ahem");
    for run in layout.glyph_runs() {
        assert_eq!(run.glyphs().len(), 0);
    }
    assert_eq!(layout.metrics().full_width(), 0.0);
    // Actual Ahem hhea800/-200/gap0, UPEM1000, size10 and the authored
    // MetricsRelative1 font strut imply height10/baseline8 despite no paint.
    assert_eq!(layout.metrics().line_count(), 1);
    assert_eq!(layout.metrics().size().height, 10.0);
    assert_eq!(layout.metrics().first_baseline(), Some(8.0));
    assert_eq!(layout.metrics().last_baseline(), Some(8.0));
}
