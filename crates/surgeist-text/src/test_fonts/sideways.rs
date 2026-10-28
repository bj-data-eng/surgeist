//! Font qualification through the existing horizontal registered-font owner.
//! The real lookup chain has distinct isolated features and deterministic
//! composition; these tests do not enable or attest sideways placement.

use super::{AHEM, cmap, layout_table, single_substitution};
use crate::test_fonts::{replace_tables, table, u16_at, u32_at};
use crate::{
    Alignment, AnalyzedSource, Direction, Font, FontCatalogueMode, Id, LineHeight, Options,
    PreparationOptions, Range, Source, Style, System, SystemOptions,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    raw::TableProvider,
};

pub(super) fn fixture() -> Vec<u8> {
    // The Hebrew entry is an authored cmap mapping to X, not a claim that
    // Ahem's licensed X design is a Hebrew glyph. It permits real odd UBA.
    let mappings = [
        (0x20, 3),
        (0x2d, 15),
        (0x41, 35),
        (0x42, 36),
        (0x43, 37),
        (0x58, 58),
        (0x5d0, 58),
    ];
    let features = vec![
        (*b"vert", 1, single_substitution(&[58], &[82])),
        (*b"vrt2", 1, single_substitution(&[58, 82], &[277, 277])),
        (
            *b"vrtr",
            1,
            single_substitution(&[58, 82, 277], &[274, 274, 274]),
        ),
    ];
    replace_tables(
        AHEM,
        vec![
            (*b"cmap", cmap(&mappings)),
            (*b"GSUB", layout_table(*b"DFLT", features)),
        ],
    )
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

#[test]
fn sideways_fixture_has_real_sfnt_cmap_and_ordered_gsub_records() {
    let bytes = super::sideways_ahem();
    assert_eq!(bytes, super::sideways_ahem());
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.maxp().unwrap().num_glyphs(), 281);
    let parsed = face.gsub().unwrap();
    assert!(parsed.script_list().is_ok());
    assert!(parsed.feature_list().is_ok());
    assert!(parsed.lookup_list().is_ok());
    let hhea = table(&bytes, *b"hhea");
    assert_eq!(i16::from_be_bytes(hhea[4..6].try_into().unwrap()), 800);
    assert_eq!(i16::from_be_bytes(hhea[6..8].try_into().unwrap()), -200);
    assert_eq!(i16::from_be_bytes(hhea[8..10].try_into().unwrap()), 0);
    for (cp, id) in [
        (0x20_u32, 3),
        (0x2d, 15),
        (0x41, 35),
        (0x42, 36),
        (0x43, 37),
        (0x58, 58),
        (0x5d0, 58),
    ] {
        assert_eq!(face.charmap().map(cp), Some(GlyphId::new(id)));
    }
    let cmap = table(&bytes, *b"cmap");
    assert_eq!(u16_at(cmap, 2), 1);
    assert_eq!(u16_at(cmap, 4), 3);
    assert_eq!(u16_at(cmap, 6), 10);
    assert_eq!(u32_at(cmap, 8), 12);
    assert_eq!(u16_at(cmap, 12), 12);
    assert_eq!(u32_at(cmap, 24), 7);
    for (index, (cp, id)) in [
        (0x20_u32, 3),
        (0x2d, 15),
        (0x41, 35),
        (0x42, 36),
        (0x43, 37),
        (0x58, 58),
        (0x5d0, 58),
    ]
    .into_iter()
    .enumerate()
    {
        let at = 28 + index * 12;
        assert_eq!(u32_at(cmap, at), cp);
        assert_eq!(u32_at(cmap, at + 4), cp);
        assert_eq!(u32_at(cmap, at + 8), id);
    }
    // Independently specified GSUB1.0: ScriptList10, FeatureList34,
    // LookupList72. DFLT's default LangSys enables feature indices0,1,2.
    let gsub = table(&bytes, *b"GSUB");
    assert_eq!(gsub.len(), 158);
    assert_eq!(&gsub[..10], &[0, 1, 0, 0, 0, 10, 0, 34, 0, 72]);
    assert_eq!(&gsub[10..18], &[0, 1, b'D', b'F', b'L', b'T', 0, 8]);
    assert_eq!(
        &gsub[18..34],
        &[0, 4, 0, 0, 0, 0, 255, 255, 0, 3, 0, 0, 0, 1, 0, 2]
    );
    assert_eq!(u16_at(gsub, 34), 3);
    for (at, tag, relative, lookup) in [
        (36, *b"vert", 20, 0),
        (42, *b"vrt2", 26, 1),
        (48, *b"vrtr", 32, 2),
    ] {
        assert_eq!(&gsub[at..at + 4], &tag);
        assert_eq!(u16_at(gsub, at + 4), relative);
        let feature = 34 + usize::from(relative);
        assert_eq!(u16_at(gsub, feature), 0);
        assert_eq!(u16_at(gsub, feature + 2), 1);
        assert_eq!(u16_at(gsub, feature + 4), lookup);
    }
    assert_eq!(u16_at(gsub, 72), 3);
    for (index, relative, expected) in [
        (0, 8, &[2, 8, 1, 82, 1, 1, 58][..]),
        (1, 30, &[2, 10, 2, 277, 277, 1, 2, 58, 82]),
        (2, 56, &[2, 12, 3, 274, 274, 274, 1, 3, 58, 82, 277]),
    ] {
        assert_eq!(u16_at(gsub, 74 + index * 2), relative);
        let lookup = 72 + usize::from(relative);
        assert_eq!(&gsub[lookup..lookup + 8], &[0, 1, 0, 0, 0, 1, 0, 8]);
        for (word, expected) in expected.iter().enumerate() {
            assert_eq!(u16_at(gsub, lookup + 8 + word * 2), *expected);
        }
    }
    for tag in [*b"glyf", *b"loca", *b"maxp", *b"hmtx", *b"hhea"] {
        assert_eq!(table(&bytes, tag), table(AHEM, tag));
    }
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

#[derive(Default)]
struct Points(Vec<(f32, f32)>);
impl OutlinePen for Points {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push((x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push((x, y));
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.0.extend([(cx, cy), (x, y)]);
    }
    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.0.extend([(a, b), (c, d), (x, y)]);
    }
    fn close(&mut self) {}
}

#[test]
fn sideways_targets_have_distinct_retained_outlines_and_horizontal_metrics() {
    let bytes = super::sideways_ahem();
    let face = skrifa::FontRef::new(&bytes).unwrap();
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    let outlines = face.outline_glyphs();
    for (id, expected) in [
        (58, [0.0, -200.0, 1000.0, 800.0]),
        (82, [0.0, -200.0, 1000.0, 0.0]),
        (277, [200.0, -200.0, 400.0, 800.0]),
        (274, [0.0, 400.0, 1000.0, 600.0]),
    ] {
        let glyph = GlyphId::new(id);
        assert_eq!(
            u16_at(table(&bytes, *b"hmtx"), usize::try_from(id).unwrap() * 4),
            1000
        );
        assert_eq!(metrics.advance_width(glyph), Some(1000.0));
        let bounds = metrics.bounds(glyph).unwrap();
        assert_eq!(
            [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
            expected
        );
        let mut points = Points::default();
        outlines
            .get(glyph)
            .unwrap()
            .draw(
                DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                &mut points,
            )
            .unwrap();
        assert!(!points.0.is_empty());
        assert_eq!(
            points.0.iter().fold(
                [
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY
                ],
                |b, &(x, y)| [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)]
            ),
            expected
        );
    }
}

fn shape(value: &str, features: &[&str], expected_id: u32, expected_level: u8) {
    let bytes = super::sideways_ahem();
    let mut font = Font::new().family("Ahem");
    for feature in features {
        font = font.feature(*feature);
    }
    let analyzed = AnalyzedSource::new(
        Source::identified(value, Id::from_u64(1515001), 7),
        Style {
            font,
            size: 10.0,
            line_height: LineHeight::Absolute(10.0),
            ..Style::default()
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        analyzed.source().text().code_units(),
        value.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(analyzed.graphemes().collect::<Vec<_>>(), [Range::new(0, 1)]);
    assert_eq!(
        analyzed.direction(),
        if expected_level == 1 {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        }
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        prepared
            .unit_views()
            .map(|u| (u.source_range(), u.level().get()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), expected_level)]
    );
    let result = system
        .layout_analyzed(
            &analyzed,
            Options {
                alignment: Alignment::Left,
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap();
    assert_eq!(result.source().id(), Some(Id::from_u64(1515001)));
    assert_eq!(result.source().revision(), 7);
    assert_eq!(
        result.source().text().code_units(),
        value.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(result.lines().len(), 1);
    assert_eq!(result.lines()[0].range(), Range::new(0, 1));
    assert_eq!(result.lines()[0].bounds().size.width, 10.0);
    let glyphs = result
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(
                run.face().catalogue_identity(),
                prepared.font_catalogue_identity()
            );
            assert_eq!(run.range(), Range::new(0, 1));
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.font_size(), 10.0);
            assert_eq!(instance.axis_count(), 0);
            run.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [(expected_id, 0.0, 0.0, 0.0, 10.0)],
        "explicit features {features:?}"
    );
    assert_eq!(
        result
            .glyph_runs()
            .iter()
            .flat_map(|r| r.glyphs().iter().map(|g| g.range()))
            .collect::<Vec<_>>(),
        [Range::new(0, 1)]
    );
}

#[test]
fn horizontal_registered_provider_observes_isolated_and_combined_features() {
    for (features, glyph) in [
        (&[][..], 58),
        (&[r#""vert" 1"#][..], 82),
        (&[r#""vrt2" 1"#][..], 277),
        (&[r#""vrtr" 1"#][..], 274),
        (&[r#""vert" 1"#, r#""vrt2" 1"#][..], 277),
        (&[r#""vert" 1"#, r#""vrtr" 1"#][..], 274),
        (&[r#""vrt2" 1"#, r#""vrtr" 1"#][..], 274),
        (&[r#""vert" 1"#, r#""vrt2" 1"#, r#""vrtr" 1"#][..], 274),
        (&[r#""vert" 1"#, r#""vrtr" 0"#][..], 82),
        (&[r#""vrt2" 1"#, r#""vrtr" 0"#][..], 277),
        (&[r#""vrtr" 1"#, r#""vrtr" 0"#][..], 58),
        (&[r#""vrtr" 0"#, r#""vrtr" 1"#][..], 274),
        (&[r#""vert" 0"#, r#""vrt2" 0"#, r#""vrtr" 0"#][..], 58),
    ] {
        shape("X", features, glyph, 0);
    }
}

#[test]
fn authored_hebrew_mapping_has_actual_odd_level_and_same_feature_chain() {
    shape("\u{5d0}", &[], 58, 1);
    shape("\u{5d0}", &[r#""vert" 1"#], 82, 1);
    shape("\u{5d0}", &[r#""vrt2" 1"#], 277, 1);
    shape("\u{5d0}", &[r#""vrtr" 1"#], 274, 1);
    shape(
        "\u{5d0}",
        &[r#""vert" 1"#, r#""vrt2" 1"#, r#""vrtr" 1"#],
        274,
        1,
    );
}
