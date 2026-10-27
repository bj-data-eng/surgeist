//! Authored static vertical-metric SFNT data over the licensed Ahem resource.
//! These helpers encode font data; no vertical-query implementation or oracle.

use crate::test_fonts::{remove_tables, replace_tables, table, u16_at};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
pub(crate) const X: u32 = 58;

pub(crate) fn absent() -> Vec<u8> {
    remove_tables(AHEM, &[*b"vhea", *b"vmtx", *b"VVAR"])
}

/// Native1200/100 metrics for every real glyph, with full long records.
pub(crate) fn native() -> Vec<u8> {
    let font = absent();
    let count = u16_at(table(&font, *b"maxp"), 4);
    with_native(&font, 1200, 100, count)
}

/// Uniform actual metrics; later advances repeat the final long metric.
pub(crate) fn with_native(font: &[u8], advance: u16, tsb: i16, long_metrics: u16) -> Vec<u8> {
    let count = usize::from(u16_at(table(font, *b"maxp"), 4));
    vertical(font, &vec![(advance, tsb); count], long_metrics)
}

pub(crate) fn repeated() -> Vec<u8> {
    let font = absent();
    let count = usize::from(u16_at(table(&font, *b"maxp"), 4));
    let mut metrics = vec![(1200, 200); count];
    metrics[0].1 = 100;
    vertical(&font, &metrics, 1)
}

pub(crate) fn zero_signed() -> Vec<u8> {
    let font = absent();
    let count = u16_at(table(&font, *b"maxp"), 4);
    let mut metrics = vec![(1200, 100); usize::from(count)];
    metrics[usize::try_from(X).unwrap()] = (0, -1000);
    vertical(&font, &metrics, count)
}

fn vertical(font: &[u8], metrics: &[(u16, i16)], long_metrics: u16) -> Vec<u8> {
    let face = skrifa::FontRef::new(font).unwrap();
    let count = face.maxp().unwrap().num_glyphs();
    assert_eq!(metrics.len(), usize::from(count));
    assert!((1..=count).contains(&long_metrics));
    let glyphs = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    let mut min_top = i16::MAX;
    let mut min_bottom = i16::MAX;
    let mut max_extent = i16::MIN;
    for (index, &(advance, tsb)) in metrics.iter().enumerate() {
        let bounds = glyphs
            .bounds(GlyphId::new(u32::try_from(index).unwrap()))
            .expect("qualified fixture glyph has an actual bounds reference");
        let height = bounds.y_max - bounds.y_min;
        assert_eq!(
            height.fract(),
            0.0,
            "these authored fixtures have integer bounds"
        );
        let height = height as i32;
        min_top = min_top.min(tsb);
        min_bottom =
            min_bottom.min(i16::try_from(i32::from(advance) - i32::from(tsb) - height).unwrap());
        max_extent = max_extent.max(i16::try_from(i32::from(tsb) + height).unwrap());
        if index >= usize::from(long_metrics) {
            assert_eq!(advance, metrics[usize::from(long_metrics) - 1].0);
        }
    }
    let mut vhea = 0x0001_0000_u32.to_be_bytes().to_vec();
    // Authored vertical typographic line metrics are independent of the
    // per-glyph advance. Fontwide used extrema below come from these sources.
    for value in [1000_i16, -1000, 0] {
        vhea.extend(value.to_be_bytes());
    }
    vhea.extend(metrics.iter().map(|m| m.0).max().unwrap().to_be_bytes());
    for value in [min_top, min_bottom, max_extent, 0, 1, 0, 0, 0, 0, 0, 0] {
        vhea.extend(value.to_be_bytes());
    }
    vhea.extend(long_metrics.to_be_bytes());
    assert_eq!(vhea.len(), 36);
    let mut vmtx = Vec::new();
    for (index, &(advance, tsb)) in metrics.iter().enumerate() {
        if index < usize::from(long_metrics) {
            vmtx.extend(advance.to_be_bytes());
        }
        vmtx.extend(tsb.to_be_bytes());
    }
    replace_tables(font, vec![(*b"vhea", vhea), (*b"vmtx", vmtx)])
}

/// Genuine absence with selected hhea metrics and an independently authored X
/// advance. This fixture deliberately clears USE_TYPO_METRICS.
pub(crate) fn horizontal(ascent: i16, descent: i16, gap: i16, x_advance: u16) -> Vec<u8> {
    assert!(ascent > descent);
    assert!(x_advance >= 1000);
    let font = absent();
    let mut hhea = table(&font, *b"hhea").to_vec();
    for (offset, value) in [(4, ascent), (6, descent), (8, gap)] {
        hhea[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    hhea[10..12].copy_from_slice(&x_advance.to_be_bytes());
    let mut os2 = table(&font, *b"OS/2").to_vec();
    let flags = u16_at(&os2, 62) & !0x80;
    os2[62..64].copy_from_slice(&flags.to_be_bytes());
    let mut hmtx = table(&font, *b"hmtx").to_vec();
    let at = usize::try_from(X).unwrap() * 4;
    hmtx[at..at + 2].copy_from_slice(&x_advance.to_be_bytes());
    replace_tables(
        &font,
        vec![(*b"hhea", hhea), (*b"OS/2", os2), (*b"hmtx", hmtx)],
    )
}

fn cff_index(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut out = u16::try_from(objects.len()).unwrap().to_be_bytes().to_vec();
    if objects.is_empty() {
        return out;
    }
    out.push(1); // all offsets in this tiny authored INDEX fit one byte
    let mut offset = 1_u8;
    out.push(offset);
    for object in objects {
        offset = offset
            .checked_add(u8::try_from(object.len()).unwrap())
            .unwrap();
        out.push(offset);
    }
    for object in objects {
        out.extend(object);
    }
    out
}
fn cff_integer(out: &mut Vec<u8>, value: i16) {
    out.push(28);
    out.extend(value.to_be_bytes());
}

/// Real two-glyph OTTO/CFF1 font: empty .notdef0 and the Ahem X rectangle1.
/// No glyf/loca or false VORG marker stands in for a CFF outline.
pub(crate) fn cff() -> Vec<u8> {
    let name = cff_index(&[b"Ahem".to_vec()]);
    let strings = cff_index(&[b"X".to_vec()]);
    let subrs = cff_index(&[]);
    let charset = vec![0, 1, 135]; // format0, custom SID391 == StringINDEX[0]
    let mut rectangle = Vec::new();
    for value in [0, -200] {
        cff_integer(&mut rectangle, value);
    }
    rectangle.push(21); // rmoveto
    for value in [1000, 0, 0, 1000, -1000, 0, 0, -1000] {
        cff_integer(&mut rectangle, value);
    }
    rectangle.extend([5, 14]); // rlineto, endchar
    let charstrings = cff_index(&[vec![14], rectangle]);
    let mut private = Vec::new();
    cff_integer(&mut private, 1000);
    private.push(20); // defaultWidthX
    cff_integer(&mut private, 0);
    private.push(21); // nominalWidthX
    // Fixed-width DICT shortint operands make these actual offsets independent
    // of their encoded values. The single TopDICT is28bytes plus INDEX5bytes.
    let charset_at = 4 + name.len() + 33 + strings.len() + subrs.len();
    let charstrings_at = charset_at + charset.len();
    let private_at = charstrings_at + charstrings.len();
    let mut dict = Vec::new();
    for value in [0, -200, 1000, 800] {
        cff_integer(&mut dict, value);
    }
    dict.push(5); // FontBBox
    cff_integer(&mut dict, i16::try_from(charset_at).unwrap());
    dict.push(15);
    cff_integer(&mut dict, i16::try_from(charstrings_at).unwrap());
    dict.push(17);
    cff_integer(&mut dict, i16::try_from(private.len()).unwrap());
    cff_integer(&mut dict, i16::try_from(private_at).unwrap());
    dict.push(18);
    assert_eq!(dict.len(), 28);
    let mut cff = vec![1, 0, 4, 4];
    for bytes in [
        name,
        cff_index(&[dict]),
        strings,
        subrs,
        charset,
        charstrings,
        private,
    ] {
        cff.extend(bytes);
    }
    let base = remove_tables(
        &absent(),
        &[
            *b"glyf", *b"loca", *b"cvt ", *b"fpgm", *b"prep", *b"gasp", *b"LTSH", *b"hdmx",
            *b"VDMX", *b"kern", *b"GSUB", *b"GPOS", *b"GDEF",
        ],
    );
    let mut hhea = table(&base, *b"hhea").to_vec();
    hhea[34..36].copy_from_slice(&2_u16.to_be_bytes());
    let mut post = table(&base, *b"post")[..32].to_vec();
    post[..4].copy_from_slice(&0x0003_0000_u32.to_be_bytes());
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12, 0, 12, 0, 0];
    for value in [28_u32, 0, 1, 88, 88, 1] {
        cmap.extend(value.to_be_bytes());
    }
    let mut font = replace_tables(
        &base,
        vec![
            (*b"CFF ", cff),
            (*b"maxp", vec![0, 0, 0x50, 0, 0, 2]),
            (*b"hhea", hhea),
            (*b"hmtx", vec![3, 232, 0, 0, 3, 232, 0, 0]),
            (*b"post", post),
            (*b"cmap", cmap),
        ],
    );
    font[..4].copy_from_slice(b"OTTO");
    // Reuse the real checksum owner after changing the SFNT flavor.
    replace_tables(&font, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FontCatalogueMode, FontData, System, SystemOptions};
    use skrifa::{
        outline::{DrawSettings, OutlineGlyphFormat, OutlinePen},
        raw::TableProvider,
    };

    fn registered(bytes: &[u8], axes: usize) -> FontData {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(bytes.to_vec()).unwrap();
        let faces = system.registered_font_faces().collect::<Vec<_>>();
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].index(), 0);
        assert_eq!(faces[0].bytes(), bytes);
        let data = faces[0].to_owned();
        drop(faces);
        assert_eq!(
            data.unsynthesized_instance(10.0, &[]).unwrap().axis_count(),
            axes
        );
        drop(system);
        assert_eq!(data.bytes(), bytes);
        data
    }
    fn bounds(font: &skrifa::FontRef<'_>, glyph: u32) -> Option<[f32; 4]> {
        font.glyph_metrics(Size::unscaled(), LocationRef::default())
            .bounds(GlyphId::new(glyph))
            .map(|b| [b.x_min, b.y_min, b.x_max, b.y_max])
    }
    fn raw_metric(font: &skrifa::FontRef<'_>, glyph: usize) -> (u16, i16) {
        let count = usize::from(font.vhea().unwrap().number_of_long_ver_metrics());
        let vmtx = font.vmtx().unwrap();
        if glyph < count {
            let row = &vmtx.v_metrics()[glyph];
            (row.advance.get(), row.side_bearing.get())
        } else {
            (
                vmtx.v_metrics().last().unwrap().advance.get(),
                vmtx.top_side_bearings()[glyph - count].get(),
            )
        }
    }
    #[test]
    fn static_native_and_absent_sources_have_real_metrics_bounds_and_registration() {
        let absent = absent();
        let face = skrifa::FontRef::new(&absent).unwrap();
        assert_eq!(face.head().unwrap().units_per_em(), 1000);
        assert_eq!(face.maxp().unwrap().num_glyphs(), 281);
        assert!(face.table_data(skrifa::Tag::new(b"vhea")).is_none());
        assert!(face.table_data(skrifa::Tag::new(b"vmtx")).is_none());
        assert_eq!(face.charmap().map(0x58_u32).unwrap().to_u32(), X);
        assert_eq!(
            face.glyph_metrics(Size::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(X)),
            Some(1000.0)
        );
        assert_eq!(bounds(&face, X), Some([0.0, -200.0, 1000.0, 800.0]));
        registered(&absent, 0);
        let native = native();
        let face = skrifa::FontRef::new(&native).unwrap();
        assert_eq!(face.vhea().unwrap().number_of_long_ver_metrics(), 281);
        assert_eq!(table(&native, *b"vmtx").len(), 1124);
        assert_eq!(raw_metric(&face, 58), (1200, 100));
        assert_eq!(bounds(&face, X), Some([0.0, -200.0, 1000.0, 800.0]));
        registered(&native, 0);
        for (a, d, gap, w) in [(900, -300, 123, 1000), (1000, -200, -100, 1001)] {
            let bytes = horizontal(a, d, gap, w);
            let face = skrifa::FontRef::new(&bytes).unwrap();
            let hhea = face.hhea().unwrap();
            assert_eq!(
                (
                    hhea.ascender().to_i16(),
                    hhea.descender().to_i16(),
                    hhea.line_gap().to_i16()
                ),
                (a, d, gap)
            );
            assert_eq!(face.os2().unwrap().fs_selection().bits() & 0x80, 0);
            assert_eq!(
                face.glyph_metrics(Size::unscaled(), LocationRef::default())
                    .advance_width(GlyphId::new(X)),
                Some(f32::from(w))
            );
            assert_eq!(bounds(&face, X), Some([0.0, -200.0, 1000.0, 800.0]));
            registered(&bytes, 0);
            let native = with_native(&bytes, 1200, 100, 281);
            assert_eq!(
                raw_metric(&skrifa::FontRef::new(&native).unwrap(), 58),
                (1200, 100)
            );
            registered(&native, 0);
        }
    }
    #[test]
    fn repeated_advance_trailing_bearings_and_zero_signed_rows_are_real() {
        let repeated = repeated();
        let face = skrifa::FontRef::new(&repeated).unwrap();
        assert_eq!(face.vhea().unwrap().number_of_long_ver_metrics(), 1);
        assert_eq!(table(&repeated, *b"vmtx").len(), 564);
        assert_eq!(raw_metric(&face, 0), (1200, 100));
        assert_eq!(raw_metric(&face, 3), (1200, 200));
        assert_eq!(raw_metric(&face, 58), (1200, 200));
        assert_eq!(raw_metric(&face, 280), (1200, 200));
        registered(&repeated, 0);
        let zero = zero_signed();
        let face = skrifa::FontRef::new(&zero).unwrap();
        assert_eq!(raw_metric(&face, 58), (0, -1000));
        assert_eq!(bounds(&face, X), Some([0.0, -200.0, 1000.0, 800.0]));
        registered(&zero, 0);
    }
    #[test]
    fn actual_empty_and_static_composite_keep_distinct_outline_references() {
        let bytes = native();
        let face = skrifa::FontRef::new(&bytes).unwrap();
        // Skrifa's raw metric supplier reports a zero bbox for the source-
        // proven empty record; this is distinct from a decoding failure.
        assert_eq!(bounds(&face, 3), Some([0.0; 4]));
        assert_eq!(raw_metric(&face, 3), (1200, 100));
        assert_eq!(face.head().unwrap().index_to_loc_format(), 0);
        let loca = table(&bytes, *b"loca");
        assert_eq!(u16_at(loca, 6), u16_at(loca, 8));
        let compound = crate::baseline_test_fonts::compound_ahem();
        let bytes = with_native(&compound, 1200, 100, 281);
        let face = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(face.charmap().map(0x41_u32).unwrap().to_u32(), 35);
        assert_eq!(bounds(&face, 35), Some([0.0, -200.0, 1000.0, 800.0]));
        assert_eq!(raw_metric(&face, 35), (1200, 100));
        registered(&bytes, 0);
    }
    #[test]
    fn actual_variable_source_has_an_axis_even_at_default_coordinates() {
        let bytes = crate::test_fonts::variable_ahem();
        let face = skrifa::FontRef::new(&bytes).unwrap();
        assert!(face.table_data(skrifa::Tag::new(b"fvar")).is_some());
        assert!(face.table_data(skrifa::Tag::new(b"gvar")).is_some());
        let data = registered(&bytes, 1);
        for coords in [&[][..], &[0][..], &[16384][..]] {
            let instance = data.unsynthesized_instance(10.0, coords).unwrap();
            assert_eq!(instance.axis_count(), 1);
            assert_eq!(
                instance.normalized_coords().collect::<Vec<_>>(),
                if coords.is_empty() {
                    vec![0]
                } else {
                    coords.to_vec()
                }
            );
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
        fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
            self.0.extend([(cx0, cy0), (cx1, cy1), (x, y)]);
        }
        fn close(&mut self) {}
    }
    #[test]
    fn genuine_cff_one_font_parses_draws_and_registers_its_type_two_rectangle() {
        let bytes = cff();
        assert_eq!(&bytes[..4], b"OTTO");
        let face = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(face.head().unwrap().units_per_em(), 1000);
        assert_eq!(face.maxp().unwrap().num_glyphs(), 2);
        assert!(face.table_data(skrifa::Tag::new(b"glyf")).is_none());
        assert!(face.table_data(skrifa::Tag::new(b"loca")).is_none());
        assert!(face.table_data(skrifa::Tag::new(b"CFF ")).is_some());
        assert_eq!(face.charmap().map(0x58_u32).unwrap().to_u32(), 1);
        assert_eq!(
            face.glyph_metrics(Size::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(1)),
            Some(1000.0)
        );
        let outlines = face.outline_glyphs();
        assert_eq!(outlines.format(), Some(OutlineGlyphFormat::Cff));
        let glyph = outlines.get(GlyphId::new(1)).unwrap();
        assert_eq!(glyph.format(), OutlineGlyphFormat::Cff);
        let mut pen = Points::default();
        glyph
            .draw(
                DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                &mut pen,
            )
            .unwrap();
        assert!(pen.0.contains(&(0.0, -200.0)));
        assert!(pen.0.contains(&(1000.0, 800.0)));
        assert!(
            pen.0
                .iter()
                .all(|&(x, y)| (0.0..=1000.0).contains(&x) && (-200.0..=800.0).contains(&y))
        );
        assert_eq!(bounds(&face, 1), Some([0.0, -200.0, 1000.0, 800.0]));
        let mut empty = Points::default();
        outlines
            .get(GlyphId::new(0))
            .unwrap()
            .draw(
                DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                &mut empty,
            )
            .unwrap();
        assert!(empty.0.is_empty());
        registered(&bytes, 0);
    }
}
