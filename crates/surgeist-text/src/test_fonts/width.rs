//! Fixture qualification through separate literal None layouts, not evidence
//! that full-width mapping or cumulative text transforms are implemented.
//! The authored SFNT cmap and hmtx are the independent glyph/advance oracle.
//! Canonical 30AB+3099 = 30AC is Unicode normalization during shaping;
//! halfwidth FF76+FF9E has only a compatibility decomposition and stays distinct.

use super::AHEM;
use crate::test_fonts::{table, u16_at, u32_at};
use crate::{
    Alignment, Direction, Font, FontCatalogueMode, Id, LineHeight, Options, Range, Source, Style,
    System, SystemOptions, TextTransform, WhiteSpace,
};
use skrifa::MetadataProvider;

#[test]
fn width_fixture_has_real_cmap_metrics_blank_spaces_and_unchanged_outlines() {
    let bytes = super::width_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let cmap = table(&bytes, *b"cmap");
    assert_eq!(&cmap[..12], &[0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12]);
    assert_eq!(u16_at(cmap, 12), 12);
    assert_eq!(u32_at(cmap, 16), 328);
    assert_eq!(u32_at(cmap, 24), 26);
    let loca = table(&bytes, *b"loca");
    let count = u16_at(table(&bytes, *b"maxp"), 4);
    assert_eq!(u16_at(table(&bytes, *b"head"), 18), 1000);
    assert_eq!(u16_at(table(&bytes, *b"head"), 50), 0);
    assert!(u16_at(table(&bytes, *b"hhea"), 34) >= count);
    for (index, (cp, glyph, advance)) in [
        (0x20_u32, 3, 500),
        (0x21, 4, 1000),
        (0x41, 35, 1000),
        (0x53, 53, 1000),
        (0x73, 85, 1000),
        (0xa2, 97, 1000),
        (0xdf, 134, 1000),
        (0x3000, 1, 2000),
        (0x3099, 100, 0),
        (0x30a1, 67, 800),
        (0x30a2, 36, 2000),
        (0x30ab, 38, 1000),
        (0x30ac, 37, 1800),
        (0x3131, 71, 1900),
        (0x3164, 70, 1700),
        (0xff01, 72, 1400),
        (0xff21, 36, 2000),
        (0xff33, 58, 1500),
        (0xff53, 86, 1250),
        (0xff67, 82, 600),
        (0xff71, 35, 1000),
        (0xff76, 82, 600),
        (0xff9e, 83, 400),
        (0xffa0, 68, 900),
        (0xffa1, 69, 1100),
        (0xffe0, 73, 1600),
    ]
    .into_iter()
    .enumerate()
    {
        // Literal format12 readback is separate from the font parser's cmap.
        let group = 28 + index * 12;
        assert_eq!(u32_at(cmap, group), cp);
        assert_eq!(u32_at(cmap, group + 4), cp);
        assert_eq!(u32_at(cmap, group + 8), glyph);
        assert_eq!(font.charmap().map(cp).unwrap().to_u32(), glyph);
        assert!(glyph > 0 && glyph < u32::from(count));
        let at = usize::try_from(glyph).unwrap();
        assert_eq!(u16_at(table(&bytes, *b"hmtx"), at * 4), advance);
        let start = u16_at(loca, at * 2);
        let end = u16_at(loca, (at + 1) * 2);
        if cp == 0x20 || cp == 0x3000 {
            assert_eq!(start, end, "space has advance without ink");
        } else {
            assert!(end > start, "covered glyph has a retained outline");
        }
    }
    for tag in [*b"glyf", *b"loca", *b"maxp", *b"hhea"] {
        assert_eq!(table(&bytes, tag), table(AHEM, tag));
    }
    let sum = |bytes: &[u8]| {
        bytes.chunks(4).fold(0_u32, |acc, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            acc.wrapping_add(u32::from_be_bytes(word))
        })
    };
    assert_eq!(sum(&bytes), 0xb1b0_afba);
    for index in 0..usize::from(u16_at(&bytes, 4)) {
        let record = 12 + index * 16;
        let tag = bytes[record..record + 4].try_into().unwrap();
        let mut data = table(&bytes, tag).to_vec();
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        assert_eq!(sum(&data), u32_at(&bytes, record + 4));
    }
}

fn baseline(text: &str, units: &[u16], expected: &[(u32, f32)], width: f64) {
    let bytes = super::width_ahem();
    let source = Source::identified(text, Id::from_u64(6303), 13);
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
                text_transform: TextTransform::none(),
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
    assert_eq!(layout.source().id(), Some(Id::from_u64(6303)));
    assert_eq!(layout.source().revision(), 13);
    assert_eq!(layout.source().text().code_units(), units);
    assert_eq!(layout.direction(), Direction::LeftToRight);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), Range::new(0, units.len()));
    assert_eq!(layout.lines()[0].bounds().size.width, width);
    let actual = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert!(!actual.is_empty());
    assert!(actual.iter().all(|(glyph, _)| *glyph != 0));
    assert_eq!(actual, expected);
}

#[test]
fn width_latin_and_casing_target_literals_have_distinct_none_metrics() {
    baseline("A", &[0x41], &[(35, 10.0)], 10.0);
    baseline("Ａ", &[0xff21], &[(36, 20.0)], 20.0);
    baseline("ß", &[0xdf], &[(134, 10.0)], 10.0);
    baseline("SS", &[0x53, 0x53], &[(53, 10.0), (53, 10.0)], 20.0);
    baseline("ＳＳ", &[0xff33, 0xff33], &[(58, 15.0), (58, 15.0)], 30.0);
    baseline("Ss", &[0x53, 0x73], &[(53, 10.0), (85, 10.0)], 20.0);
    baseline("Ｓｓ", &[0xff33, 0xff53], &[(58, 15.0), (86, 12.5)], 27.5);
}

#[test]
fn width_space_literals_retain_blank_ink_and_distinct_nonzero_advances() {
    // Interior spaces expose advance in line bounds; trailing whitespace is
    // excluded from those bounds by the existing line-layout contract.
    baseline(
        "A A",
        &[0x41, 0x20, 0x41],
        &[(35, 10.0), (3, 5.0), (35, 10.0)],
        25.0,
    );
    baseline(
        "A　A",
        &[0x41, 0x3000, 0x41],
        &[(35, 10.0), (1, 20.0), (35, 10.0)],
        40.0,
    );
}

#[test]
fn width_kana_literals_separate_width_size_and_canonical_composition() {
    baseline("ｧ", &[0xff67], &[(82, 6.0)], 6.0);
    baseline("ｱ", &[0xff71], &[(35, 10.0)], 10.0);
    baseline("ァ", &[0x30a1], &[(67, 8.0)], 8.0);
    baseline("ア", &[0x30a2], &[(36, 20.0)], 20.0);
    baseline("ｶﾞ", &[0xff76, 0xff9e], &[(82, 6.0), (83, 4.0)], 10.0);
    baseline("カ", &[0x30ab], &[(38, 10.0)], 10.0);
    baseline("ガ", &[0x30ac], &[(37, 18.0)], 18.0);
    baseline("カ\u{3099}", &[0x30ab, 0x3099], &[(37, 18.0)], 18.0);
}

#[test]
fn width_hangul_punctuation_and_currency_literals_have_covered_none_glyphs() {
    baseline("\u{ffa0}", &[0xffa0], &[(68, 9.0)], 9.0);
    baseline("\u{3164}", &[0x3164], &[(70, 17.0)], 17.0);
    baseline("\u{ffa1}", &[0xffa1], &[(69, 11.0)], 11.0);
    baseline("\u{3131}", &[0x3131], &[(71, 19.0)], 19.0);
    baseline("!", &[0x21], &[(4, 10.0)], 10.0);
    baseline("！", &[0xff01], &[(72, 14.0)], 14.0);
    baseline("¢", &[0xa2], &[(97, 10.0)], 10.0);
    baseline("￠", &[0xffe0], &[(73, 16.0)], 16.0);
}
