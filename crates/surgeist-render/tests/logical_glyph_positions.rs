//! Functional qualification of logical f64 glyph placement at the real outline boundary.
use surgeist_render::{FontData, GlyphRunLimits, GlyphSynthesis, Rect, TextGlyph, TextInstance};

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

#[test]
fn logical_glyph_position_is_retained_until_actual_path_point_conversion() {
    // At this origin, adjacent f32 positions are two units apart. Ahem's
    // literal one-unit-wide square must be translated in f64 before conversion:
    // ties-to-even maps 2^24+1 to 2^24, while the right edge 2^24+2 is exact.
    // Narrowing the origin first would collapse both sides to the same point.
    let origin = 16_777_217.0_f64;
    let glyph = TextGlyph::try_new(58, origin, 0.0, -3.0).unwrap();
    assert_eq!(glyph.x(), origin);
    assert_eq!(glyph.advance(), -3.0);
    let font = FontData::try_from_bytes(AHEM.to_vec(), 0).unwrap();
    let instance = TextInstance::try_new(1.0, vec![], GlyphSynthesis::none()).unwrap();
    let prepared = font
        .prepare_glyph_run(
            &instance,
            &[glyph],
            GlyphRunLimits::new(1, 5, 1 << 20, 1 << 20),
        )
        .unwrap();
    assert_eq!(prepared.glyphs(), [glyph]);
    let top = f64::from(-0.8_f32);
    let bottom = f64::from(0.2_f32);
    assert_eq!(
        prepared.bounds().ink_rect(),
        Some(Rect::try_new(16_777_216.0, top, 2.0, bottom - top).unwrap())
    );
}

#[test]
fn logical_positions_preserve_f64_domain_and_reject_nonfinite_values() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(TextGlyph::try_new(58, value, 0.0, 0.0).is_err());
        assert!(TextGlyph::try_new(58, 0.0, value, 0.0).is_err());
    }
    for value in [f64::MAX, -f64::MAX, f64::from_bits(1), -f64::from_bits(1)] {
        let glyph = TextGlyph::try_new(58, value, -value, 0.0).unwrap();
        assert_eq!(glyph.x(), value);
        assert_eq!(glyph.y(), -value);
    }
    assert!(TextGlyph::try_new(58, 0.0, 0.0, f32::NAN).is_err());
}
