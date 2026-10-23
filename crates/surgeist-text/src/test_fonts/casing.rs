//! Literal font baselines, always using TextTransform::none().
//!
//! Casing expectations are independent literal Unicode mappings: sharp-s upper
//! SS/title Ss; default dotted-I lower i+0307 versus Turkic i; Lithuanian dot
//! insertion/removal; Greek accent/dialytika and final/nonfinal sigma. These
//! separate original strings do not execute or certify those transformations.
//!
//! Glyph-normalization expectations follow Unicode canonical normalization
//! (UAX #15), rather than the casing algorithm or observed glyph output:
//! I+0307 = 0130, I+0301 = 00CD,
//! i+0301 = 00ED, and Greek IOTA+0308 = 03AA. The existing composed-diacritics
//! shaper selects those supported cmap forms. Dot/acute both have CCC230, so
//! i+0307+0301 cannot compose across the intervening dot; 03AA has no canonical
//! composition with acute. Original Source units remain unchanged in every case.

use super::AHEM;
use crate::test_fonts::{table, u16_at, u32_at};
use crate::{
    Alignment, Direction, Font, FontCatalogueMode, Id, LineHeight, Options, Range, Source, Style,
    System, SystemOptions, TextTransform, WhiteSpace,
};
use skrifa::MetadataProvider;

#[test]
fn casing_fixture_has_real_distinct_cmap_metrics_outlines_and_checksums() {
    let bytes = super::casing_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let original = skrifa::FontRef::new(AHEM).unwrap();
    for cp in [
        0x130_u32, 0x301, 0x307, 0x308, 0x386, 0x390, 0x391, 0x399, 0x39f, 0x3a3, 0x3aa, 0x3ac,
        0x3b9, 0x3bf, 0x3c2, 0x3c3,
    ] {
        assert!(original.charmap().map(cp).is_none());
    }
    for (cp, glyph) in [
        (0x49_u32, 43),
        (0x69, 75),
        (0x130, 36),
        (0x131, 178),
        (0xcd, 168),
        (0xed, 115),
        (0x301, 102),
        (0x307, 100),
        (0x308, 101),
        (0x386, 154),
        (0x389, 165),
        (0x390, 110),
        (0x391, 35),
        (0x394, 210),
        (0x397, 36),
        (0x399, 43),
        (0x39a, 45),
        (0x39f, 49),
        (0x3a3, 53),
        (0x3aa, 55),
        (0x3ac, 105),
        (0x3ae, 106),
        (0x3af, 107),
        (0x3b1, 67),
        (0x3b4, 70),
        (0x3b7, 71),
        (0x3b9, 75),
        (0x3ba, 77),
        (0x3bf, 81),
        (0x3c2, 84),
        (0x3c3, 85),
        (0x3ca, 109),
    ] {
        assert_eq!(font.charmap().map(cp).unwrap().to_u32(), glyph);
        let at = usize::try_from(glyph).unwrap();
        let advance = if (100..=102).contains(&glyph) {
            0
        } else {
            1000
        };
        assert_eq!(u16_at(table(&bytes, *b"hmtx"), at * 4), advance);
        let loca = table(&bytes, *b"loca");
        // Licensed Ahem uses short loca offsets. These are real nonempty glyphs.
        assert!(u16_at(loca, (at + 1) * 2) > u16_at(loca, at * 2));
    }
    assert_eq!(u16_at(table(&bytes, *b"head"), 18), 1000);
    assert_eq!(u16_at(table(&bytes, *b"head"), 50), 0);
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

fn baseline(bytes: Vec<u8>, text: &str, units: &[u16], expected: &[(u32, f32)], width: f64) {
    let source = Source::identified(text, Id::from_u64(6302), 12);
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
    assert_eq!(layout.source().id(), Some(Id::from_u64(6302)));
    assert_eq!(layout.source().revision(), 12);
    assert_eq!(layout.source().text().code_units(), units);
    assert_eq!(layout.direction(), Direction::LeftToRight);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), Range::new(0, units.len()));
    assert_eq!(layout.lines()[0].bounds().size.width, width);
    let glyphs = layout
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
    assert!(!glyphs.is_empty());
    assert!(glyphs.iter().all(|(glyph, _)| *glyph != 0));
    assert_eq!(glyphs, expected);
    for run in layout.run_views() {
        for glyph in run
            .glyphs()
            .filter(|glyph| (100..=102).contains(&glyph.id()))
        {
            // The fixture's actual GPOS: (400-100,600-200)/1000*10.
            assert_eq!(run.origin().x + glyph.pen() + f64::from(glyph.x()), 3.0);
            assert_eq!(glyph.y(), -4.0);
        }
    }
}

#[test]
fn original_ahem_sharp_s_upper_and_title_targets_have_distinct_real_glyphs() {
    baseline(AHEM.to_vec(), "ß", &[0xdf], &[(134, 10.0)], 10.0);
    baseline(
        AHEM.to_vec(),
        "SS",
        &[0x53, 0x53],
        &[(53, 10.0), (53, 10.0)],
        20.0,
    );
    baseline(
        AHEM.to_vec(),
        "Ss",
        &[0x53, 0x73],
        &[(53, 10.0), (85, 10.0)],
        20.0,
    );
    baseline(
        AHEM.to_vec(),
        "Iiı",
        &[0x49, 0x69, 0x131],
        &[(43, 10.0), (75, 10.0), (178, 10.0)],
        30.0,
    );
}

#[test]
fn dotted_i_source_default_expansion_and_turkic_target_are_real_none_layouts() {
    baseline(super::casing_ahem(), "İ", &[0x130], &[(36, 10.0)], 10.0);
    baseline(
        super::casing_ahem(),
        "i\u{307}",
        &[0x69, 0x307],
        &[(75, 10.0), (100, 0.0)],
        10.0,
    );
    baseline(
        super::casing_ahem(),
        "I\u{307}",
        &[0x49, 0x307],
        &[(36, 10.0)],
        10.0,
    );
    baseline(super::casing_ahem(), "i", &[0x69], &[(75, 10.0)], 10.0);
}

#[test]
fn lithuanian_source_dot_insertion_and_removal_targets_keep_real_marks() {
    baseline(
        super::casing_ahem(),
        "I\u{301}",
        &[0x49, 0x301],
        &[(168, 10.0)],
        10.0,
    );
    baseline(
        super::casing_ahem(),
        "i\u{307}\u{301}",
        &[0x69, 0x307, 0x301],
        &[(75, 10.0), (100, 0.0), (102, 0.0)],
        10.0,
    );
    baseline(
        super::casing_ahem(),
        "I\u{307}\u{301}",
        &[0x49, 0x307, 0x301],
        &[(36, 10.0), (102, 0.0)],
        10.0,
    );
    baseline(
        super::casing_ahem(),
        "i\u{301}",
        &[0x69, 0x301],
        &[(115, 10.0)],
        10.0,
    );
}

#[test]
fn greek_three_scalar_upper_target_uses_covered_canonical_dialytika_and_mark() {
    baseline(super::casing_ahem(), "ΐ", &[0x390], &[(110, 10.0)], 10.0);
    baseline(
        super::casing_ahem(),
        "Ι\u{308}\u{301}",
        &[0x399, 0x308, 0x301],
        &[(55, 10.0), (102, 0.0)],
        10.0,
    );
}

#[test]
fn greek_accent_dialytika_and_isolated_eta_literals_have_distinct_real_glyphs() {
    for (text, units, glyphs) in [
        ("άι", &[0x3ac, 0x3b9][..], &[(105, 10.0), (75, 10.0)][..]),
        ("ΑΪ", &[0x391, 0x3aa], &[(35, 10.0), (55, 10.0)]),
        ("ΆΙ", &[0x386, 0x399], &[(154, 10.0), (43, 10.0)]),
        ("ή", &[0x3ae], &[(106, 10.0)]),
        ("Ή", &[0x389], &[(165, 10.0)]),
    ] {
        baseline(
            super::casing_ahem(),
            text,
            units,
            glyphs,
            10.0 * glyphs.len() as f64,
        );
    }
}

#[test]
fn greek_final_and_nonfinal_sigma_sources_and_targets_are_distinct_none_layouts() {
    for (text, units, glyphs) in [
        ("ΟΣ", &[0x39f, 0x3a3][..], &[(49, 10.0), (53, 10.0)][..]),
        ("ος", &[0x3bf, 0x3c2], &[(81, 10.0), (84, 10.0)]),
        (
            "ΟΣΑ",
            &[0x39f, 0x3a3, 0x391],
            &[(49, 10.0), (53, 10.0), (35, 10.0)],
        ),
        (
            "οσα",
            &[0x3bf, 0x3c3, 0x3b1],
            &[(81, 10.0), (85, 10.0), (67, 10.0)],
        ),
    ] {
        baseline(
            super::casing_ahem(),
            text,
            units,
            glyphs,
            10.0 * glyphs.len() as f64,
        );
    }
}
