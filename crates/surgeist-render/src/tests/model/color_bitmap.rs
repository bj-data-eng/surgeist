//! Authored Ahem derivatives qualify actual selected color/image representations.
//! The original licensed fixture is unchanged. Literal geometry follows its
//! UPEM1000 X square (0,-200)..(1000,800), not glyph advance rectangles.

use super::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};

#[cfg(feature = "render-text")]
#[path = "color_bitmap/gpu.rs"]
mod gpu;

fn limits() -> crate::GlyphRunLimits {
    crate::GlyphRunLimits::new(4, 1024, 8 << 20, 8 << 20)
}

fn instance(size: f32) -> crate::TextInstance {
    crate::TextInstance::try_new(size, vec![], crate::GlyphSynthesis::none()).unwrap()
}

fn assert_retained_ink(bytes: Vec<u8>, size: f32, expected: Rect) {
    let original = bytes.clone();
    let font = FontData::try_from_bytes(bytes, 0).unwrap();
    let instance = instance(size);
    let glyphs = vec![TextGlyph::try_new(AHEM_GLYPH_X, 3.0, 11.0, 777.0).unwrap()];
    let result = font.prepare_glyph_run(&instance, &glyphs, limits());
    assert!(
        result.is_ok(),
        "selected representation must admit its real ink: {result:?}"
    );
    let ready = result.unwrap();
    assert_eq!(ready.instance(), &instance);
    assert_eq!(ready.glyphs(), glyphs);
    drop(glyphs);
    drop(instance);
    drop(font);
    assert_eq!(ready.font().bytes(), original);
    assert_eq!(ready.font().index(), 0);
    assert_eq!(ready.bounds().kind(), TextRunBoundsKind::Ink);
    assert_eq!(ready.bounds().ink_rect(), Some(expected));
    assert!(ready.is_same_preparation(&ready.clone()));
}

#[test]
fn selected_colr_v0_retains_literal_red_square_ink_at_actual_instance() {
    let bytes = ahem_color_font(valid_cpal());
    verify_color(&bytes, false);
    // Glyph baseline (3,11), size10: x=3..13, y=11-8..11+2.
    assert_retained_ink(bytes, 10.0, Rect::try_new(3.0, 3.0, 10.0, 10.0).unwrap());
}

#[test]
fn selected_colr_v1_executes_true_solid_glyph_translation_and_clip() {
    let bytes = translated_clipped_v1();
    verify_color(&bytes, true);
    // +200,+100 font-unit translation gives x5..15,y2..12 at size10.
    // The root clip (0,0)..(800,800) gives x3..11,y3..11: intersection6x8.
    assert_retained_ink(bytes, 10.0, Rect::try_new(5.0, 3.0, 6.0, 8.0).unwrap());
}

#[test]
fn selected_sbix_png_uses_exact_strike_offsets_and_retains_image_ink() {
    let bytes = offset_sbix();
    verify_bitmap(&bytes);
    // Cached Skrifa0.42.1's Skia/CoreText convention: X outer bearing
    // (LSB0,yMin-200) scales to (0,-4); selected20ppem inner bearing(2,3)
    // and BottomLeft origin give x=3+2=5, top=11-(-4)-3-1=11.
    assert_retained_ink(bytes, 20.0, Rect::try_new(5.0, 11.0, 1.0, 1.0).unwrap());
}

fn translated_clipped_v1() -> Vec<u8> {
    let mut colr = Vec::new();
    push_be_u16(&mut colr, 1);
    push_be_u16(&mut colr, 0);
    push_be_u32(&mut colr, 0);
    push_be_u32(&mut colr, 0);
    push_be_u16(&mut colr, 0);
    push_be_u32(&mut colr, 34); // BaseGlyphList
    push_be_u32(&mut colr, 0); // LayerList absent
    push_be_u32(&mut colr, 63); // ClipList
    push_be_u32(&mut colr, 0); // variation index map absent
    push_be_u32(&mut colr, 0); // variation store absent
    push_be_u32(&mut colr, 1);
    push_be_u16(&mut colr, AHEM_GLYPH_X.try_into().unwrap());
    push_be_u32(&mut colr, 10); // paint relative to BaseGlyphList
    colr.extend_from_slice(&[14, 0, 0, 8]); // PaintTranslate -> PaintGlyph
    push_be_u16(&mut colr, 200);
    push_be_u16(&mut colr, 100);
    colr.extend_from_slice(&[10, 0, 0, 6]); // PaintGlyph -> PaintSolid
    push_be_u16(&mut colr, AHEM_GLYPH_X.try_into().unwrap());
    colr.push(2); // PaintSolid, actual opaque red palette0
    push_be_u16(&mut colr, 0);
    push_be_u16(&mut colr, 0x4000);
    colr.push(1); // ClipList format1
    push_be_u32(&mut colr, 1);
    push_be_u16(&mut colr, AHEM_GLYPH_X.try_into().unwrap());
    push_be_u16(&mut colr, AHEM_GLYPH_X.try_into().unwrap());
    colr.extend_from_slice(&[0, 0, 12]); // clip box relative to ClipList
    colr.push(1); // ClipBox format1
    for coordinate in [0, 0, 800, 800] {
        push_be_u16(&mut colr, coordinate);
    }
    ahem_with_tables(vec![(*b"COLR", colr), (*b"CPAL", valid_cpal())])
}

fn offset_sbix() -> Vec<u8> {
    let mut sbix = Vec::new();
    push_be_u16(&mut sbix, 1);
    push_be_u16(&mut sbix, 1);
    push_be_u32(&mut sbix, 3);
    sbix.resize(20, 0);
    for (index, (ppem, pixel)) in [
        (10, [0, 255, 0, 255]),
        (20, [255, 0, 0, 255]),
        (30, [0, 0, 255, 255]),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = u32::try_from(sbix.len()).unwrap();
        write_be_u32(&mut sbix, 8 + index * 4, offset);
        let png = encoded_png(png::ColorType::Rgba, &pixel);
        let mut strike = sbix_strike(&png, SbixStrikeFixture::new(ppem, true));
        let record = 4 + (ahem_num_glyphs() + 1) * 4;
        write_be_u16(&mut strike, record, 2);
        write_be_u16(&mut strike, record + 2, 3);
        sbix.extend(strike);
    }
    ahem_with_tables(vec![(*b"sbix", sbix)])
}

#[cfg(feature = "render-text")]
fn mixed_representation_font() -> Vec<u8> {
    // A remains an ordinary outline, X selects COLRv0, B selects sbix. The
    // bitmap record is moved to B by its actual per-glyph offset interval.
    let mut sbix = sbix_table(&rgba_png(), &[SbixStrikeFixture::new(16, true)]);
    let strike = read_be_u32(&sbix, 8) as usize;
    let record = 4 + (ahem_num_glyphs() + 1) * 4;
    let end = sbix.len() - strike;
    for glyph in 0..=ahem_num_glyphs() {
        write_be_u32(
            &mut sbix,
            strike + 4 + glyph * 4,
            u32::try_from(if glyph <= 36 { record } else { end }).unwrap(),
        );
    }
    font_with_tables(&ahem_color_font(valid_cpal()), vec![(*b"sbix", sbix)])
}

#[derive(Default)]
struct ColorFacts {
    glyphs: Vec<u32>,
    solids: Vec<(u16, f32)>,
    transforms: Vec<[f32; 6]>,
    clips: Vec<[f32; 4]>,
}

impl skrifa::color::ColorPainter for ColorFacts {
    fn push_transform(&mut self, transform: skrifa::color::Transform) {
        self.transforms.push([
            transform.xx,
            transform.yx,
            transform.xy,
            transform.yy,
            transform.dx,
            transform.dy,
        ]);
    }
    fn pop_transform(&mut self) {}
    fn push_clip_glyph(&mut self, glyph: GlyphId) {
        self.glyphs.push(glyph.to_u32());
    }
    fn push_clip_box(&mut self, bounds: skrifa::raw::types::BoundingBox<f32>) {
        self.clips
            .push([bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max]);
    }
    fn pop_clip(&mut self) {}
    fn fill(&mut self, brush: skrifa::color::Brush<'_>) {
        let skrifa::color::Brush::Solid {
            palette_index,
            alpha,
        } = brush
        else {
            panic!("authored fixture has only solid paints")
        };
        self.solids.push((palette_index, alpha));
    }
    fn push_layer(&mut self, _: skrifa::color::CompositeMode) {
        panic!("authored fixture has no composites")
    }
}

fn verify_color(bytes: &[u8], v1: bool) {
    let font = skrifa::FontRef::from_index(bytes, 0).unwrap();
    let colr = font.colr().unwrap();
    if v1 {
        assert_eq!(colr.num_base_glyph_records(), 0);
        assert!(
            colr.v1_base_glyph(GlyphId::new(AHEM_GLYPH_X))
                .unwrap()
                .is_some()
        );
    }
    let collection = font.color_glyphs();
    let glyph = collection.get(GlyphId::new(AHEM_GLYPH_X)).unwrap();
    assert_eq!(
        matches!(glyph.format(), skrifa::color::ColorGlyphFormat::ColrV1),
        v1
    );
    let mut facts = ColorFacts::default();
    glyph
        .paint(skrifa::instance::LocationRef::default(), &mut facts)
        .unwrap();
    assert_eq!(facts.glyphs, [AHEM_GLYPH_X]);
    assert_eq!(facts.solids, [(0, 1.0)]);
    if v1 {
        assert_eq!(facts.transforms, [[1.0, 0.0, 0.0, 1.0, 200.0, 100.0]]);
        assert_eq!(facts.clips, [[0.0, 0.0, 800.0, 800.0]]);
    }
    let palettes = font.color_palettes();
    let palette = palettes.get(0).unwrap();
    let red = palette.colors()[0];
    assert_eq!(
        (red.red(), red.green(), red.blue(), red.alpha()),
        (255, 0, 0, 255)
    );
}

fn verify_bitmap(bytes: &[u8]) {
    let font = skrifa::FontRef::from_index(bytes, 0).unwrap();
    let strikes = font.bitmap_strikes();
    assert_eq!(strikes.len(), 3);
    let glyph = strikes
        .glyph_for_size(
            skrifa::instance::Size::new(20.0),
            GlyphId::new(AHEM_GLYPH_X),
        )
        .unwrap();
    assert_eq!((glyph.ppem_x, glyph.ppem_y), (20.0, 20.0));
    assert_eq!((glyph.bearing_x, glyph.bearing_y), (0.0, -200.0));
    assert_eq!((glyph.inner_bearing_x, glyph.inner_bearing_y), (2.0, 3.0));
    assert_eq!((glyph.width, glyph.height), (1, 1));
    assert!(matches!(
        glyph.placement_origin,
        skrifa::bitmap::Origin::BottomLeft
    ));
    let skrifa::bitmap::BitmapData::Png(png) = glyph.data else {
        panic!("actual selected source is PNG")
    };
    let mut reader = png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .unwrap();
    let mut decoded = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut decoded).unwrap();
    assert_eq!((info.width, info.height), (1, 1));
    assert_eq!(&decoded[..info.buffer_size()], &[255, 0, 0, 255]);
}

#[test]
fn authored_colr_v0_fixture_executes_real_red_glyph_paint() {
    verify_color(&ahem_color_font(valid_cpal()), false);
}

#[test]
fn authored_colr_v1_fixture_executes_real_translation_clip_and_red_glyph_paint() {
    verify_color(&translated_clipped_v1(), true);
}

#[test]
fn authored_sbix_fixture_selects_middle_strike_offsets_and_decodes_red_pixel() {
    verify_bitmap(&offset_sbix());
}

#[cfg(feature = "render-text")]
fn assert_registered_adapter(bytes: Vec<u8>, size: f32, ink: Rect) {
    use surgeist_text as text;
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let input = system
        .layout(
            text::Source::new("X"),
            text::Style {
                font: text::Font::new().family("Ahem"),
                size,
                line_height: text::LineHeight::Absolute(f64::from(size)),
                // Intrinsic red color/image pixels must not become this blue
                // foreground. GPU qualification follows headless admission.
                brush: text::Brush::color(0.0, 0.0, 1.0, 1.0),
                ..text::Style::default()
            },
            text::Options {
                quantize: false,
                ..text::Options::default()
            },
        )
        .unwrap();
    assert_eq!(input.source().text().code_units(), &[88]);
    let run = input.run_views().next().unwrap();
    assert_eq!(run.glyphs().next().unwrap().id(), AHEM_GLYPH_X);
    assert_eq!(run.face().bytes(), bytes);
    assert_eq!(run.effective_instance().unwrap().font_size(), size);
    drop(system);
    let placement = Transform::translation(3.0, 11.0).unwrap();
    let result = crate::prepare_text_layout(
        &input,
        placement,
        crate::TextLayoutLimits::new(limits(), 4, 4, 4),
    );
    assert!(
        result.is_ok(),
        "actual registered selected representation must admit: {result:?}"
    );
    let prepared = result.unwrap();
    assert!(std::ptr::eq(prepared.layout(), &input));
    let scene = prepared.scene().clone();
    drop(prepared);
    drop(input);
    let [
        scene::Command::PreparedGlyphRun {
            run: outline,
            transform,
            ..
        },
    ] = scene.commands.as_slice()
    else {
        panic!("one actual run must retain one admitted paint command: {scene:?}");
    };
    assert_eq!(outline.font().bytes(), bytes);
    assert_eq!(outline.instance().size(), size);
    assert_eq!(outline.bounds().ink_rect(), Some(ink));
    assert_eq!(
        *transform,
        Transform::translation(3.0, 11.0 + f64::from(size) * 0.8).unwrap()
    );
}

#[cfg(feature = "render-text")]
#[test]
fn registered_colr_v0_layout_admits_same_face_instance_and_retained_ink() {
    let bytes = ahem_color_font(valid_cpal());
    verify_color(&bytes, false);
    assert_registered_adapter(bytes, 10.0, Rect::try_new(0.0, -8.0, 10.0, 10.0).unwrap());
}

#[cfg(feature = "render-text")]
#[test]
fn registered_true_colr_v1_layout_admits_translated_clipped_ink() {
    let bytes = translated_clipped_v1();
    verify_color(&bytes, true);
    assert_registered_adapter(bytes, 10.0, Rect::try_new(2.0, -8.0, 6.0, 8.0).unwrap());
}

#[cfg(feature = "render-text")]
#[test]
fn registered_sbix_png_layout_admits_selected_strike_bearing_ink() {
    let bytes = offset_sbix();
    verify_bitmap(&bytes);
    assert_registered_adapter(bytes, 20.0, Rect::try_new(2.0, 0.0, 1.0, 1.0).unwrap());
}

fn table_bytes(bytes: &[u8], tag: &[u8; 4]) -> Vec<u8> {
    let font = skrifa::FontRef::from_index(bytes, 0).unwrap();
    font.table_data(skrifa::Tag::new(tag))
        .unwrap()
        .as_bytes()
        .to_vec()
}

fn one_glyph() -> [TextGlyph; 1] {
    [TextGlyph::try_new(AHEM_GLYPH_X, 3.0, 11.0, 777.0).unwrap()]
}

fn refusal(bytes: Vec<u8>, instance: &crate::TextInstance) -> crate::GlyphPreparationError {
    let error = FontData::try_from_bytes(bytes, 0)
        .unwrap()
        .prepare_glyph_run(instance, &one_glyph(), limits())
        .unwrap_err();
    *error.glyph_preparation_diagnostic().unwrap()
}

#[test]
fn transparent_selected_color_and_fully_decoded_png_prove_empty_without_advance_ink() {
    let mut palette = valid_cpal();
    *palette.last_mut().unwrap() = 0;
    for bytes in [
        ahem_color_font(palette),
        ahem_sbix_font(encoded_png(png::ColorType::Rgba, &[255, 0, 0, 0])),
    ] {
        let ready = FontData::try_from_bytes(bytes, 0)
            .unwrap()
            .prepare_glyph_run(&instance(20.0), &one_glyph(), limits())
            .unwrap();
        assert_eq!(ready.bounds().kind(), TextRunBoundsKind::Empty);
        assert_eq!(ready.glyphs()[0].advance(), 777.0);
        assert_eq!(ready.paints().len(), 0);
    }
}

#[test]
fn unsupported_valid_colr_root_solid_refuses_before_reinterpreting_it_as_glyph_paint() {
    let bytes = translated_clipped_v1();
    let mut colr = table_bytes(&bytes, b"COLR");
    // A valid clipped root PaintSolid is not the selected PaintGlyph subset.
    colr[44..49].copy_from_slice(&[2, 0, 0, 0x40, 0]);
    let bytes = font_with_tables(&bytes, vec![(*b"COLR", colr)]);
    let font = skrifa::FontRef::from_index(&bytes, 0).unwrap();
    let mut facts = ColorFacts::default();
    font.color_glyphs()
        .get(GlyphId::new(AHEM_GLYPH_X))
        .unwrap()
        .paint(skrifa::instance::LocationRef::default(), &mut facts)
        .unwrap();
    assert!(facts.glyphs.is_empty());
    assert_eq!(facts.solids, [(0, 1.0)]);
    assert_eq!(
        refusal(bytes, &instance(10.0)),
        crate::GlyphPreparationError::UnsupportedColorPaint { format: 2 }
    );
}

#[test]
fn malformed_selected_v1_root_never_falls_back_to_a_valid_v0_root() {
    let bytes = translated_clipped_v1();
    let mut colr = table_bytes(&bytes, b"COLR");
    let v0_offset = u32::try_from(colr.len()).unwrap();
    push_be_u16(&mut colr, AHEM_GLYPH_X.try_into().unwrap());
    push_be_u16(&mut colr, 0);
    push_be_u16(&mut colr, 1);
    let layers = u32::try_from(colr.len()).unwrap();
    push_be_u16(&mut colr, AHEM_GLYPH_X.try_into().unwrap());
    push_be_u16(&mut colr, 0);
    write_be_u16(&mut colr, 2, 1);
    write_be_u32(&mut colr, 4, v0_offset);
    write_be_u32(&mut colr, 8, layers);
    write_be_u16(&mut colr, 12, 1);
    write_be_u32(&mut colr, 40, u32::MAX);
    let bytes = font_with_tables(&bytes, vec![(*b"COLR", colr)]);
    let font = skrifa::FontRef::from_index(&bytes, 0).unwrap();
    assert!(
        font.colr()
            .unwrap()
            .v0_base_glyph(GlyphId::new(AHEM_GLYPH_X))
            .unwrap()
            .is_some()
    );
    assert!(
        font.colr()
            .unwrap()
            .v1_base_glyph(GlyphId::new(AHEM_GLYPH_X))
            .is_err()
    );
    assert_eq!(
        refusal(bytes, &instance(10.0)),
        crate::GlyphPreparationError::InvalidFont
    );
}

fn with_axis(bytes: &[u8]) -> Vec<u8> {
    let mut fvar = Vec::new();
    push_be_u32(&mut fvar, 0x0001_0000);
    for value in [16, 2, 1, 20, 0, 8] {
        push_be_u16(&mut fvar, value);
    }
    fvar.extend_from_slice(b"wght");
    for value in [0, 0, 65536] {
        push_be_u32(&mut fvar, value);
    }
    push_be_u16(&mut fvar, 0);
    push_be_u16(&mut fvar, 256);
    font_with_tables(bytes, vec![(*b"fvar", fvar)])
}

#[test]
fn default_axis_is_retained_but_selected_image_variation_and_synthesis_refuse() {
    for bytes in [
        ahem_color_font(valid_cpal()),
        translated_clipped_v1(),
        offset_sbix(),
    ] {
        let variable = with_axis(&bytes);
        let zero = crate::TextInstance::try_new(
            20.0,
            vec![crate::NormalizedCoordinate::try_from_bits(0).unwrap()],
            crate::GlyphSynthesis::none(),
        )
        .unwrap();
        let ready = FontData::try_from_bytes(variable.clone(), 0)
            .unwrap()
            .prepare_glyph_run(&zero, &one_glyph(), limits())
            .unwrap();
        assert_eq!(ready.instance(), &zero);
        let varied = crate::TextInstance::try_new(
            20.0,
            vec![crate::NormalizedCoordinate::try_from_bits(16384).unwrap()],
            crate::GlyphSynthesis::none(),
        )
        .unwrap();
        assert_eq!(
            refusal(variable, &varied),
            crate::GlyphPreparationError::UnsupportedImageInstance
        );
        for synthesis in [
            crate::GlyphSynthesis::try_new(0.2, 0.0).unwrap(),
            crate::GlyphSynthesis::try_new(0.0, 1.0).unwrap(),
        ] {
            let selected = crate::TextInstance::try_new(20.0, vec![], synthesis).unwrap();
            assert_eq!(
                refusal(bytes.clone(), &selected),
                crate::GlyphPreparationError::UnsupportedImageInstance
            );
        }
    }
}

#[test]
fn selected_png_encoding_and_truncation_refuse_without_outline_substitution() {
    assert_eq!(
        refusal(ahem_sbix_font(grayscale_png()), &instance(16.0)),
        crate::GlyphPreparationError::UnsupportedPng
    );
    assert_eq!(
        refusal(ahem_sbix_font(malformed_png()), &instance(16.0)),
        crate::GlyphPreparationError::InvalidFont
    );
    assert_eq!(
        refusal(ahem_color_font(invalid_cpal()), &instance(16.0)),
        crate::GlyphPreparationError::InvalidFont
    );
}

#[test]
fn selected_color_and_png_limits_are_atomic_and_same_input_retries_remain_valid() {
    for bytes in [
        ahem_color_font(valid_cpal()),
        translated_clipped_v1(),
        offset_sbix(),
    ] {
        let font = FontData::try_from_bytes(bytes.clone(), 0).unwrap();
        let instance = instance(20.0);
        for tight in [
            crate::GlyphRunLimits::new(0, 1024, 8 << 20, 8 << 20),
            crate::GlyphRunLimits::new(4, 1024, 0, 8 << 20),
            crate::GlyphRunLimits::new(4, 1024, 8 << 20, 0),
        ] {
            let error = font
                .prepare_glyph_run(&instance, &one_glyph(), tight)
                .unwrap_err();
            assert!(
                matches!(
                    error.glyph_preparation_diagnostic(),
                    Some(crate::GlyphPreparationError::Limit { .. })
                ),
                "{error:?}"
            );
            assert_eq!(font.bytes(), bytes);
            let ready = font
                .prepare_glyph_run(&instance, &one_glyph(), limits())
                .unwrap();
            assert_eq!(ready.instance(), &instance);
            assert_eq!(ready.bounds().kind(), TextRunBoundsKind::Ink);
        }
    }
}

#[test]
fn mixed_foreground_and_intrinsic_color_preserve_actual_overlap_order() {
    let font = FontData::try_from_bytes(ahem_color_font(valid_cpal()), 0).unwrap();
    let glyphs = [35, AHEM_GLYPH_X, 35].map(|id| TextGlyph::try_new(id, 0.0, 8.0, 10.0).unwrap());
    let ready = font
        .prepare_glyph_run(&instance(10.0), &glyphs, limits())
        .unwrap();
    // These are actual executable payload colors and identical overlapping
    // contour geometry: foreground, opaque red, then foreground again.
    let colors = ready
        .paints()
        .iter()
        .map(|operation| match operation {
            crate::text::GlyphPaint::Fill { color, .. } => *color,
            _ => panic!("this fixture contains no bitmap"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        colors,
        [
            None,
            Some(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
            None
        ]
    );
    assert_eq!(
        ready.bounds().ink_rect(),
        Some(Rect::try_new(0.0, 0.0, 10.0, 10.0).unwrap())
    );
    for operation in ready.paints() {
        let crate::text::GlyphPaint::Fill { elements, .. } = operation else {
            unreachable!()
        };
        let path = kurbo::BezPath::from_vec(ready.path().elements()[elements.clone()].to_vec());
        assert_eq!(
            kurbo::Shape::bounding_box(&path),
            kurbo::Rect::new(0.0, 0.0, 10.0, 10.0)
        );
    }
}

#[test]
fn bitmap_without_paths_admits_zero_path_capacity_but_charges_decoded_pixel_storage() {
    let tiny = FontData::try_from_bytes(ahem_sbix_font(rgba_png()), 0).unwrap();
    let cap = tiny.bytes().len() + 1024;
    let ready = tiny
        .prepare_glyph_run(
            &instance(16.0),
            &one_glyph(),
            crate::GlyphRunLimits::new(1, 0, 8 << 20, cap),
        )
        .unwrap();
    assert_eq!(ready.path_elements().count(), 0);
    assert_eq!(ready.bounds().kind(), TextRunBoundsKind::Ink);
    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, 32, 32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[255, 0, 0, 255].repeat(32 * 32))
            .unwrap();
    }
    let large = FontData::try_from_bytes(ahem_sbix_font(png), 0).unwrap();
    let error = large
        .prepare_glyph_run(
            &instance(16.0),
            &one_glyph(),
            crate::GlyphRunLimits::new(1, 0, 8 << 20, cap),
        )
        .unwrap_err();
    assert!(
        matches!(
            error.glyph_preparation_diagnostic(),
            Some(crate::GlyphPreparationError::Limit {
                resource: crate::GlyphResource::RetainedBytes,
                ..
            })
        ),
        "{error:?}"
    );
    let retried = large
        .prepare_glyph_run(&instance(16.0), &one_glyph(), limits())
        .unwrap();
    assert_eq!(retried.bounds().ink_rect().unwrap().width(), 32.0);
    // At16ppem, original yMin-200 yields bottom11+3.2=14.2 and
    // top14.2-32=-17.8. Canonical endpoints each convert once to f32.
    assert_eq!(
        retried.bounds().ink_rect().unwrap().height(),
        f64::from(14.2_f32) - f64::from(-17.8_f32)
    );
    let mut scene = Scene::new();
    scene.prepared_glyph_run(
        retried.clone(),
        Transform::identity(),
        TextPaint::try_fill(Color::BLACK.into()).unwrap(),
    );
    let commands = scene.normalize(Capabilities::CURRENT).unwrap();
    let encoded =
        crate::encode::encode_vello_scene_with_initial_transform(&commands, Transform::identity())
            .unwrap();
    let paths = encoded
        .observation_for_test()
        .path_geometry_for_test()
        .unwrap();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].0, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    use crate::vello_engine::scene::VelloPathElementObservationForTest as Element;
    let points = paths[0]
        .2
        .iter()
        .filter_map(|element| match element {
            Element::MoveTo(point) | Element::LineTo(point) => Some(*point),
            Element::Close => None,
            _ => panic!("actual retained bitmap quad must remain rectangular"),
        })
        .collect::<Vec<_>>();
    assert!(
        points
            .iter()
            .all(|point| [3.0, 35.0].contains(&point[0])
                && [-17.8_f32, 14.2_f32].contains(&point[1])),
        "{points:?}"
    );
    for expected in [
        [3.0, -17.8_f32],
        [35.0, -17.8_f32],
        [35.0, 14.2_f32],
        [3.0, 14.2_f32],
    ] {
        assert!(
            points.contains(&expected),
            "missing actual admitted endpoint {expected:?}: {points:?}"
        );
    }
}
