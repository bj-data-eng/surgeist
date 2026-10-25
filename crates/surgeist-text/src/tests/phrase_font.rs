//! Qualifies real fixed-advance coverage for the independently sourced phrase
//! fixtures. Phrase segmentation itself is documented in the font provenance;
//! these tests do not infer a linguistic expectation from Text's implementation.

use crate::*;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
};

const ORIGINAL: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
const PHRASE_FONT: &[u8] = include_bytes!("../../tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf");
const FAMILY: &str = "Surgeist Phrase Ahem";
const FULL: &str = "日本へようこそ";
const UNITS: [u16; 7] = [0x65e5, 0x672c, 0x3078, 0x3088, 0x3046, 0x3053, 0x305d];

fn style() -> Style {
    Style {
        font: Font::new().family(FAMILY),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        locale: Some("ja".into()),
        wrap: Wrap::None,
        ..Style::default()
    }
}

fn system() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    // Both faces coexist, so the requested derivative must select its own
    // family/resource even where original Ahem could otherwise supply fallback.
    system.register_fonts(ORIGINAL.to_vec()).unwrap();
    system.register_fonts(PHRASE_FONT.to_vec()).unwrap();
    system
}

#[test]
fn phrase_font_has_real_japanese_cmap_and_preserves_the_qualified_square() {
    let font = skrifa::FontRef::new(PHRASE_FONT).unwrap();
    let original = skrifa::FontRef::new(ORIGINAL).unwrap();
    assert_eq!(FULL.encode_utf16().collect::<Vec<_>>(), UNITS);
    assert_eq!(FULL.chars().count(), 7);
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    assert_eq!(
        font.metrics(FontSize::unscaled(), LocationRef::default())
            .units_per_em,
        1000
    );
    for ch in FULL.chars() {
        assert_eq!(original.charmap().map(ch), None, "original cmap: {ch}");
        assert_eq!(font.charmap().map(ch), Some(GlyphId::new(35)), "{ch}");
        assert_eq!(metrics.advance_width(GlyphId::new(35)), Some(1000.0));
    }
    assert_eq!(font.charmap().map('漢'), None, "coverage remains narrow");
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map(' '), Some(GlyphId::new(3)));
    for tag in [*b"glyf", *b"loca", *b"hmtx", *b"hhea", *b"maxp"] {
        assert_eq!(
            crate::test_fonts::table(PHRASE_FONT, tag),
            crate::test_fonts::table(ORIGINAL, tag),
            "actual original outline and metrics: {tag:?}"
        );
    }
}

#[test]
fn phrase_font_native_preparation_retains_each_source_unit_and_real_instance() {
    let mut system = system();
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        let source = Source::identified(FULL, Id::from_u64(65065), 7);
        let request = ParagraphInput {
            source: source.clone(),
            base_style: style(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: direction,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, 7),
                aggregate: Range::new(0, 7),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        };
        let analyzed = AnalyzedSource::from_paragraph(request).unwrap();
        let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::None);
        assert_eq!(analyzed.source(), &source);
        assert_eq!(analyzed.source().text().code_units(), UNITS);
        assert_eq!(
            analyzed.processed_source().processed_text().code_units(),
            UNITS
        );
        assert_eq!(
            analyzed.graphemes().collect::<Vec<_>>(),
            (0..7).map(|i| Range::new(i, i + 1)).collect::<Vec<_>>()
        );
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
        assert_eq!(prepared.analyzed_source().source(), &source);
        let units = prepared.unit_views().collect::<Vec<_>>();
        assert_eq!(units.len(), 7);
        for (i, unit) in units.iter().enumerate() {
            assert_eq!(unit.source_range(), Range::new(i, i + 1));
            assert_eq!(unit.advance(), 10.0);
            let instances = unit.font_instances().collect::<Result<Vec<_>>>().unwrap();
            assert_eq!(instances.len(), 1);
            assert_eq!(instances[0].face().bytes(), PHRASE_FONT);
            assert_eq!(instances[0].font_size(), 10.0);
        }
    }
}

#[test]
fn phrase_font_registered_shaping_proves_literal_glyphs_and_exact_advances() {
    let mut system = system();
    // The isolated input has separately qualified original-reference inference;
    // it is not assumed to inherit the full sentence's contextual prediction.
    for (text, units, width) in [("日本へ", &UNITS[..3], 30.0), (FULL, &UNITS[..], 70.0)] {
        let source = Source::identified(text, Id::from_u64(65066), 9);
        let layout = system
            .layout(
                source.clone(),
                style(),
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(layout.source(), &source);
        assert_eq!(layout.source().text().code_units(), units);
        assert_eq!(layout.metrics().line_count(), 1);
        assert_eq!(layout.metrics().full_width(), width);
        let mut actual = Vec::new();
        for run in layout.glyph_runs() {
            assert_eq!(run.font().bytes(), PHRASE_FONT);
            assert_eq!(run.font().index(), 0);
            for glyph in run.glyphs() {
                actual.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        }
        actual.sort_by_key(|glyph| glyph.0.start);
        assert_eq!(
            actual,
            (0..units.len())
                .map(|i| (Range::new(i, i + 1), 35, 10.0))
                .collect::<Vec<_>>()
        );
    }
}
