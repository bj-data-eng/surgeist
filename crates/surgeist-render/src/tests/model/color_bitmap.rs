//! Authored Ahem derivatives qualify actual selected color/image representations.
//! The original licensed fixture is unchanged. Literal geometry follows its
//! UPEM1000 X square (0,-200)..(1000,800), not glyph advance rectangles.

use super::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};

fn limits() -> crate::OutlineLimits {
    crate::OutlineLimits::new(4, 1024, 8 << 20, 8 << 20)
}

fn instance(size: f32) -> crate::TextInstance {
    crate::TextInstance::try_new(size, vec![], crate::OutlineSynthesis::none()).unwrap()
}

fn assert_retained_ink(bytes: Vec<u8>, size: f32, expected: Rect) {
    let original = bytes.clone();
    let font = FontData::try_from_bytes(bytes, 0).unwrap();
    let instance = instance(size);
    let glyphs = vec![TextGlyph::try_new(AHEM_GLYPH_X, 3.0, 11.0, 777.0).unwrap()];
    let result = font.prepare_outline(&instance, &glyphs, limits());
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
        scene::Command::PreparedOutline {
            outline, transform, ..
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
