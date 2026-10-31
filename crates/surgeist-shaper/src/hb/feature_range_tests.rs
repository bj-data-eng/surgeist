//! Regression qualification of the existing Feature::new range boundary.
//! Authored SFNT operands and parser controls establish half-open execution.
use crate::hb::completion_fixture::{gsub_lookup, with_gsub, words};
use crate::{Direction, Feature, GlyphBuffer, Shaper, ShapingStatus, Tag, UnicodeBuffer};
use alloc::vec::Vec;
use read_fonts::{types::GlyphId, TableProvider};

const A: u16 = 35;
const B: u16 = 36;

fn with_liga(test: impl FnOnce(&Shaper<'_>)) {
    // DFLT default LangSys -> liga -> lookup0: A,A becomes B.
    let mut gsub = gsub_lookup(4, &words(&[1, 18, 1, 8, 1, 4, B, 2, A, 1, 1, A]));
    gsub[32..36].copy_from_slice(b"liga");
    with_gsub(&gsub, |font| {
        assert_eq!(font.units_per_em(), 1000);
        assert_eq!(font.get_nominal_glyph('A' as u32).unwrap().to_u32(), 35);
        assert_eq!(font.get_nominal_glyph('B' as u32).unwrap().to_u32(), 36);
        let metrics = font.font.hmtx().unwrap();
        assert_eq!(metrics.advance(GlyphId::new(35)), Some(1000));
        assert_eq!(metrics.advance(GlyphId::new(36)), Some(1000));
        test(font);
    });
}

fn shape(font: &Shaper<'_>, features: &[Feature]) -> GlyphBuffer {
    let mut input = UnicodeBuffer::new();
    for ordinal in 0..4 {
        input.add('A', ordinal);
    }
    input.set_direction(Direction::LeftToRight);
    input.set_script(crate::script::LATIN);
    font.shape(input, features)
}

fn complete_glyphs(result: &GlyphBuffer) -> Vec<u32> {
    assert_eq!(result.shaping_status(), ShapingStatus::Complete);
    assert!(result.glyph_positions().iter().all(|position| {
        position.x_advance == 1000
            && position.y_advance == 0
            && position.x_offset == 0
            && position.y_offset == 0
    }));
    result
        .glyph_infos()
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect()
}

#[test]
fn half_open_constructor_disables_exact_first_scalar_like_parsed_range() {
    with_liga(|font| {
        // Both real A,A ligatures apply with ordinary provider defaults.
        assert_eq!(complete_glyphs(&shape(font, &[])), [36, 36]);
        let parsed = "liga[0:1]=0".parse::<Feature>().unwrap();
        // Scalar0 stays A; the next pair at scalar1/scalar2 becomes B.
        assert_eq!(complete_glyphs(&shape(font, &[parsed])), [35, 36, 35]);
        let constructed = Feature::new(Tag::new(b"liga"), 0, 0..1);
        assert_eq!(complete_glyphs(&shape(font, &[constructed])), [35, 36, 35]);
    });
}

#[test]
fn half_open_constructor_enables_both_ligature_components_like_parsed_range() {
    with_liga(|font| {
        assert_eq!(complete_glyphs(&shape(font, &[])), [36, 36]);
        let disabled = "liga=0".parse::<Feature>().unwrap();
        assert_eq!(complete_glyphs(&shape(font, &[disabled])), [35, 35, 35, 35]);
        let parsed = "liga[0:2]=1".parse::<Feature>().unwrap();
        // Only the first pair has both components enabled under [0,2).
        assert_eq!(
            complete_glyphs(&shape(font, &[disabled, parsed])),
            [36, 35, 35]
        );
        let constructed = Feature::new(Tag::new(b"liga"), 1, 0..2);
        assert_eq!(
            complete_glyphs(&shape(font, &[disabled, constructed])),
            [36, 35, 35]
        );
    });
}
