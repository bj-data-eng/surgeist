//! Fixed kana cmap/advance qualification over the retained licensed Ahem font.
//! These baselines shape separate original sources with TextTransform::None.

use crate::{
    Alignment, Direction, Font, FontCatalogueMode, Id, LineHeight, Options, Range, Source, Style,
    System, SystemOptions, TextTransform, WhiteSpace,
};
use skrifa::MetadataProvider;

use super::AHEM;
use crate::test_fonts::{table, u16_at};

#[test]
fn fixed_kana_cmap_and_advances_retain_original_outline_data() {
    let bytes = crate::shaping_test_fonts::kana_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let original = skrifa::FontRef::new(AHEM).unwrap();
    assert!(original.charmap().map(0x1b132_u32).is_none());
    assert!(original.charmap().map(0x3053_u32).is_none());
    for (cp, glyph) in [(0x41_u32, 35), (0x43, 37), (0x1b132, 82), (0x3053, 36)] {
        assert_eq!(font.charmap().map(cp).unwrap().to_u32(), glyph);
    }
    assert_eq!(u16_at(table(&bytes, *b"head"), 18), 1000);
    let hmtx = table(&bytes, *b"hmtx");
    for glyph in [35, 37, 82] {
        assert_eq!(u16_at(hmtx, glyph * 4), 1000);
    }
    assert_eq!(u16_at(hmtx, 36 * 4), 500);
    assert_eq!(u16_at(table(AHEM, *b"hmtx"), 36 * 4), 1000);
    for tag in [*b"glyf", *b"loca", *b"maxp", *b"hhea"] {
        assert_eq!(table(&bytes, tag), table(AHEM, tag));
    }
}

fn baseline(text: &str, units: &[u16], expected: &[(u32, f32, f64, Range)], width: f64) {
    let bytes = crate::shaping_test_fonts::kana_ahem();
    let source = Source::identified(text, Id::from_u64(6301), 11);
    assert_eq!(source.text().code_units(), units);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let layout = system
        .layout(
            source.clone(),
            Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                line_height: LineHeight::Absolute(10.0),
                text_transform: TextTransform::None,
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
            Options {
                alignment: Alignment::Left,
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap();
    drop(system);
    assert_eq!(layout.source(), &source);
    assert_eq!(layout.source().id(), Some(Id::from_u64(6301)));
    assert_eq!(layout.source().revision(), 11);
    assert_eq!(layout.source().text().code_units(), units);
    assert_eq!(layout.direction(), Direction::LeftToRight);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), Range::new(0, units.len()));
    assert_eq!(layout.lines()[0].bounds().size.width, width);
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), bytes.as_slice());
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
    }
    let actual: Vec<_> = layout
        .glyph_runs()
        .iter()
        .flat_map(|run| {
            run.glyphs()
                .iter()
                .map(|glyph| {
                    (
                        glyph.id(),
                        glyph.advance(),
                        run.origin().x + glyph.pen() + f64::from(glyph.x()),
                        glyph.range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn original_supplementary_small_kana_shapes_real_p_with_both_source_units() {
    baseline(
        "A\u{1b132}C",
        &[0x41, 0xd82c, 0xdd32, 0x43],
        &[
            (35, 10.0, 0.0, Range::new(0, 1)),
            (82, 10.0, 10.0, Range::new(1, 3)),
            (37, 10.0, 20.0, Range::new(3, 4)),
        ],
        30.0,
    );
}

#[test]
fn separate_original_full_size_kana_shapes_real_half_advance_b() {
    baseline(
        "A\u{3053}C",
        &[0x41, 0x3053, 0x43],
        &[
            (35, 10.0, 0.0, Range::new(0, 1)),
            (36, 5.0, 10.0, Range::new(1, 2)),
            (37, 10.0, 15.0, Range::new(2, 3)),
        ],
        25.0,
    );
}
