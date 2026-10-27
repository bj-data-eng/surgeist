//! Actual fallback ink must share its base's pen reference in both TTB APIs.
use super::*;
use read_fonts::types::GlyphId;

fn ink_bounds(font: &Shaper<'_>, gid: u16) -> (i32, i32, i32, i32) {
    let head = font.font.head().unwrap();
    let loca = font
        .font
        .loca(Some(head.index_to_loc_format() != 0))
        .unwrap();
    let glyf = font.font.glyf().unwrap();
    let glyph = loca
        .get_glyf(GlyphId::new(u32::from(gid)), &glyf)
        .unwrap()
        .unwrap();
    (
        i32::from(glyph.x_min()),
        i32::from(glyph.y_min()),
        i32::from(glyph.x_max()),
        i32::from(glyph.y_max()),
    )
}

fn qualify_font_and_horizontal_pen(font: &Shaper<'_>, odd: bool) {
    qualify(font, true, odd);
    assert!(font.font.gpos().is_err());
    assert!(font.font.gsub().is_err());
    assert!(font.font.maxp().unwrap().num_glyphs() > font::MARK);
    assert_eq!(
        font.get_nominal_glyph(0x300).unwrap().to_u32(),
        u32::from(font::MARK)
    );
    assert_eq!(ink_bounds(font, font::A), (0, 0, 1000, 800));
    assert_eq!(ink_bounds(font, font::MARK), (0, 0, 100, 200));
    let mark_metrics = font::metrics(font, u32::from(font::MARK));
    assert_eq!(
        (
            mark_metrics.advance(),
            mark_metrics.origin_x_twice(),
            mark_metrics.origin_y()
        ),
        (400, 200, 150)
    );
    let horizontal = ordinary(font, "A\u{300}", Direction::LeftToRight, &[]);
    assert_eq!(horizontal.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        horizontal
            .glyph_infos()
            .iter()
            .map(|glyph| (glyph.glyph_id, glyph.cluster))
            .collect::<Vec<_>>(),
        [(35, 0), (100, 0)]
    );
    assert!(horizontal.glyph_infos()[1].is_mark());
    let advance = if odd { 1001 } else { 1000 };
    assert_eq!(
        old_positions(&horizontal),
        [(advance, 0, 0, 0), (0, 0, 450 - advance, 862)]
    );
    // The mark's stored H offset is relative to the pen AFTER the base. For
    // odd width it is−551, not450; adding actual pen1001 restores ink450..550.
    let base = &horizontal.glyph_positions()[0];
    let mark = &horizontal.glyph_positions()[1];
    let mark_ink_start = base.x_advance + mark.x_offset;
    assert_eq!((mark_ink_start, mark_ink_start + 100), (450, 550));
    assert_eq!((base.x_offset, base.x_offset + 1000), (0, 1000));
    assert_eq!(mark_ink_start * 2 + 100, base.x_offset * 2 + 1000);
}

fn ordinary_ttb(odd: bool) {
    font::with_font(true, odd, None, None, &[], |font| {
        qualify_font_and_horizontal_pen(font, odd);
        let result = ordinary(font, "A\u{300}", Direction::TopToBottom, &[]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            result
                .glyph_infos()
                .iter()
                .map(|glyph| (glyph.glyph_id, glyph.cluster))
                .collect::<Vec<_>>(),
            [(35, 0), (100, 0)]
        );
        assert!(result.glyph_infos()[1].is_mark());
        let base = &result.glyph_positions()[0];
        let mark = &result.glyph_positions()[1];
        assert_eq!(
            (base.x_advance, base.y_advance, base.x_offset, base.y_offset),
            (0, -1200, -500, -900)
        );
        assert_eq!(
            (mark.x_advance, mark.y_advance, mark.y_offset),
            (0, 0, 1181)
        );
        // Raw font centering is trunc((W−100)/2)=450 for BOTH widths. The
        // ordinary nominal x origin is−trunc(W/2)=−500. These are the same
        // actual TTB pen, so translated mark ink must be−50..50, center0.
        let mark_pen_x = base.x_advance;
        let actual_ink = (mark_pen_x + mark.x_offset, mark_pen_x + mark.x_offset + 100);
        assert_eq!(
            actual_ink,
            (-50, 50),
            "ordinary vertical mark ink must include the actual base origin"
        );
        assert_eq!(mark.x_offset, -50);
        assert_eq!(actual_ink.0 * 2 + 100, base.x_offset * 2 + 1000);
    });
}

fn supplied_ttb(odd: bool) {
    font::with_font(true, odd, None, None, &[], |font| {
        qualify_font_and_horizontal_pen(font, odd);
        let result = vertical(font, "A\u{300}", &[]);
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(ids(&result), [(35, 0), (100, 0)]);
        assert!(result.glyph_infos()[1].is_mark());
        let mut positions = result.glyph_positions();
        assert_eq!(positions.len(), 2);
        let base = positions.next().unwrap();
        let mark = positions.next().unwrap();
        let expected_base = if odd { -500.5 } else { -500.0 };
        assert_eq!(
            (base.x_advance, base.y_advance, base.x_offset, base.y_offset),
            (0.0, -1200.0, expected_base, -900.0)
        );
        assert_eq!(
            (mark.x_advance, mark.y_advance, mark.y_offset),
            (0.0, 0.0, 1181.0)
        );
        // At W1001, base ink is−500.5..499.5, center−0.5. Raw quotient450
        // stays an integer; add the actual retained half-origin ONCE to obtain
        // mark−50.5..49.5. Even width has base−500 and mark−50..50.
        let expected_ink = if odd { (-50.5, 49.5) } else { (-50.0, 50.0) };
        let actual_ink = (
            base.x_advance + mark.x_offset,
            base.x_advance + mark.x_offset + 100.0,
        );
        assert_eq!(
            actual_ink, expected_ink,
            "supplied vertical mark ink must retain its base's half-unit origin"
        );
        assert_eq!(mark.x_offset.to_bits(), expected_ink.0.to_bits());
        assert_eq!(
            (actual_ink.0 * 2.0 + 100.0).to_bits(),
            (base.x_offset * 2.0 + 1000.0).to_bits()
        );
    });
}

#[test]
fn ordinary_ttb_odd_width_fallback_mark_uses_actual_base_ink_origin() {
    ordinary_ttb(true);
}

#[test]
fn ordinary_ttb_even_width_fallback_mark_uses_actual_base_ink_origin() {
    ordinary_ttb(false);
}

#[test]
fn supplied_ttb_odd_width_fallback_mark_keeps_actual_half_origin() {
    supplied_ttb(true);
}

#[test]
fn supplied_ttb_even_width_fallback_mark_uses_actual_base_ink_origin() {
    supplied_ttb(false);
}
