//! Physical font qualification with independently supplied literal None text.
//! These tests do not implement or attest a mathematical text transformation.
//! Only cmap/hmtx are authored; Ahem's public-domain/CC0-fallback outlines and
//! licensing remain unchanged (tests/fixtures/fonts/ahem/PROVENANCE.md).

use super::AHEM;
use crate::test_fonts::{replace_tables, table, u16_at, u32_at};
use crate::{
    Alignment, Font, FontCatalogueMode, Id, Layout, LineHeight, Options, Range, Source, Style,
    System, SystemOptions, TextTransform, WhiteSpace,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
};

pub(super) fn fixture() -> Vec<u8> {
    let mappings: [(u32, u32); 15] = [
        (0x20, 3),
        (0x41, 35),
        (0x63, 69),
        (0x68, 74),
        (0x131, 178),
        (0x3b1, 67),
        (0x3f4, 68),
        (0x210e, 100),
        (0x2202, 70),
        (0x1d434, 36),
        (0x1d450, 82),
        (0x1d6a4, 277),
        (0x1d6f3, 37),
        (0x1d6fc, 274),
        (0x1d715, 38),
    ];
    // One Windows Unicode-full-repertoire format12 subtable, singleton groups.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12, 0, 12, 0, 0];
    cmap.extend(196_u32.to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(15_u32.to_be_bytes());
    for (cp, glyph) in mappings {
        cmap.extend(cp.to_be_bytes());
        cmap.extend(cp.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let mut hmtx = table(AHEM, *b"hmtx").to_vec();
    for (glyph, advance) in [
        (3, 500_u16),
        (36, 2000),
        (82, 1500),
        (100, 1800),
        (277, 1700),
        (274, 1600),
        (37, 1400),
        (38, 1300),
    ] {
        hmtx[glyph * 4..glyph * 4 + 2].copy_from_slice(&advance.to_be_bytes());
    }
    replace_tables(AHEM, vec![(*b"cmap", cmap), (*b"hmtx", hmtx)])
}

#[test]
fn math_fixture_cmap_metrics_outlines_and_checksums_are_real_sfnt_data() {
    let bytes = super::math_ahem();
    assert_eq!(bytes, super::math_ahem(), "construction is deterministic");
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let cmap = table(&bytes, *b"cmap");
    assert_eq!(
        &cmap[..16],
        &[0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12, 0, 12, 0, 0]
    );
    assert_eq!(u32_at(cmap, 16), 196);
    assert_eq!(u32_at(cmap, 24), 15);
    assert_eq!(u16_at(table(&bytes, *b"head"), 18), 1000);
    assert_eq!(u16_at(table(&bytes, *b"head"), 50), 0);
    let count = u16_at(table(&bytes, *b"maxp"), 4);
    assert!(u16_at(table(&bytes, *b"hhea"), 34) >= count);
    for (index, (cp, glyph, advance)) in [
        (0x20_u32, 3, 500),
        (0x41, 35, 1000),
        (0x63, 69, 1000),
        (0x68, 74, 1000),
        (0x131, 178, 1000),
        (0x3b1, 67, 1000),
        (0x3f4, 68, 1000),
        (0x210e, 100, 1800),
        (0x2202, 70, 1000),
        (0x1d434, 36, 2000),
        (0x1d450, 82, 1500),
        (0x1d6a4, 277, 1700),
        (0x1d6f3, 37, 1400),
        (0x1d6fc, 274, 1600),
        (0x1d715, 38, 1300),
    ]
    .into_iter()
    .enumerate()
    {
        let group = 28 + index * 12;
        assert_eq!(u32_at(cmap, group), cp);
        assert_eq!(u32_at(cmap, group + 4), cp);
        assert_eq!(u32_at(cmap, group + 8), glyph);
        assert_eq!(font.charmap().map(cp).unwrap().to_u32(), glyph);
        assert!(glyph > 0 && glyph < u32::from(count));
        let at = usize::try_from(glyph).unwrap();
        assert_eq!(u16_at(table(&bytes, *b"hmtx"), at * 4), advance);
        let loca = table(&bytes, *b"loca");
        let start = u16_at(loca, at * 2);
        let end = u16_at(loca, (at + 1) * 2);
        if cp == 0x20 {
            assert_eq!(start, end);
        } else {
            assert!(end > start);
        }
    }
    for tag in [*b"glyf", *b"loca", *b"maxp", *b"hhea"] {
        assert_eq!(table(&bytes, tag), table(AHEM, tag));
    }
    let checksum = |bytes: &[u8]| {
        bytes.chunks(4).fold(0_u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    };
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    for index in 0..usize::from(u16_at(&bytes, 4)) {
        let record = 12 + index * 16;
        let tag = bytes[record..record + 4].try_into().unwrap();
        let mut data = table(&bytes, tag).to_vec();
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        assert_eq!(checksum(&data), u32_at(&bytes, record + 4));
    }
}

fn layout(text: &str, width: Option<f64>) -> Layout {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(super::math_ahem()).unwrap();
    system
        .layout(
            Source::identified(text, Id::from_u64(6304), 14),
            Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                line_height: LineHeight::Absolute(10.0),
                text_transform: TextTransform::none(),
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
            Options {
                width,
                alignment: Alignment::Left,
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap()
}

fn baseline(text: &str, units: &[u16], glyph: u32, advance: f32, ink: [f32; 4]) {
    let result = layout(text, None);
    assert_eq!(result.source().id(), Some(Id::from_u64(6304)));
    assert_eq!(result.source().revision(), 14);
    assert_eq!(result.source().text().code_units(), units);
    assert_eq!(result.lines().len(), 1);
    assert_eq!(result.lines()[0].range(), Range::new(0, units.len()));
    assert_eq!(result.lines()[0].bounds().size.width, f64::from(advance));
    let bytes = super::math_ahem();
    let glyphs = result
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            let font = skrifa::FontRef::new(run.face().bytes()).unwrap();
            let metrics = font.glyph_metrics(Size::new(10.0), LocationRef::default());
            run.glyphs()
                .map(|placed| {
                    let bounds = metrics.bounds(GlyphId::new(placed.id())).unwrap();
                    // Allow subpixel provider rounding; the expected geometry
                    // comes from retained Ahem outlines at size10/UPEM1000.
                    for (actual, expected) in
                        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max]
                            .into_iter()
                            .zip(ink)
                    {
                        assert!((actual - expected).abs() < 1.0 / 1024.0);
                    }
                    (placed.id(), placed.advance())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(glyph, advance)]);
    let ranges = result
        .glyph_runs()
        .iter()
        .flat_map(|run| run.glyphs().iter().map(|g| g.range()))
        .collect::<Vec<_>>();
    assert_eq!(ranges, [Range::new(0, units.len())]);
}

#[test]
fn math_original_literals_have_real_none_source_glyphs() {
    for (text, units, glyph) in [
        ("A", &[0x41][..], 35),
        ("c", &[0x63], 69),
        ("h", &[0x68], 74),
        ("ı", &[0x131], 178),
        ("α", &[0x3b1], 67),
        ("ϴ", &[0x3f4], 68),
        ("∂", &[0x2202], 70),
    ] {
        baseline(text, units, glyph, 10.0, [0.0, -2.0, 10.0, 8.0]);
    }
}

#[test]
fn math_target_literals_have_distinct_none_advances_and_retained_ink() {
    baseline(
        "\u{1d434}",
        &[0xd835, 0xdc34],
        36,
        20.0,
        [0.0, -2.0, 10.0, 8.0],
    );
    baseline(
        "\u{1d450}",
        &[0xd835, 0xdc50],
        82,
        15.0,
        [0.0, -2.0, 10.0, 0.0],
    );
    baseline("ℎ", &[0x210e], 100, 18.0, [0.0, 0.0, 10.0, 8.0]);
    baseline(
        "\u{1d6a4}",
        &[0xd835, 0xdea4],
        277,
        17.0,
        [2.0, -2.0, 4.0, 8.0],
    );
    baseline(
        "\u{1d6fc}",
        &[0xd835, 0xdefc],
        274,
        16.0,
        [0.0, 4.0, 10.0, 6.0],
    );
    baseline(
        "\u{1d6f3}",
        &[0xd835, 0xdef3],
        37,
        14.0,
        [0.0, -2.0, 10.0, 8.0],
    );
    baseline(
        "\u{1d715}",
        &[0xd835, 0xdf15],
        38,
        13.0,
        [0.0, -2.0, 10.0, 8.0],
    );
}

#[test]
fn math_literal_advances_drive_distinct_wrapping_with_blank_interior_space() {
    let original = layout("A A", Some(30.0));
    assert_eq!(original.lines().len(), 1);
    assert_eq!(layout("A A", None).lines()[0].bounds().size.width, 25.0);
    let target = layout("\u{1d434} \u{1d434}", Some(30.0));
    assert_eq!(
        target.source().text().code_units(),
        &[0xd835, 0xdc34, 0x20, 0xd835, 0xdc34]
    );
    assert_eq!(target.lines().len(), 2);
    assert_eq!(target.lines()[0].range(), Range::new(0, 3));
    assert_eq!(target.lines()[1].range(), Range::new(3, 5));
    let unwrapped = layout("\u{1d434} \u{1d434}", None);
    assert_eq!(unwrapped.lines()[0].bounds().size.width, 45.0);
    let bytes = super::math_ahem();
    let glyphs = unwrapped
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            let font = skrifa::FontRef::new(run.face().bytes()).unwrap();
            // Skrifa reports an empty glyf entry as a present default box.
            let space = font
                .glyph_metrics(Size::new(10.0), LocationRef::default())
                .bounds(GlyphId::new(3))
                .unwrap();
            assert_eq!(
                [space.x_min, space.y_min, space.x_max, space.y_max],
                [0.0; 4]
            );
            run.glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(36, 20.0), (3, 5.0), (36, 20.0)]);
}
