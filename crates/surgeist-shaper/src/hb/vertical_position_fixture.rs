//! Independently authored static SFNT operands for supplied vertical metrics.
//! Retains the licensed Ahem resource; all changed tables are real SFNT tables.
use crate::hb::completion_fixture::{with_tables, words};
use crate::{Shaper, VerticalNominalMetrics};
use alloc::{vec, vec::Vec};
use read_fonts::{types::GlyphId, TableProvider};

pub(super) const A: u16 = 35;
pub(super) const B: u16 = 36;
pub(super) const R: u16 = 58;
pub(super) const MARK: u16 = 100;
pub(super) const MARK2: u16 = 101;
pub(super) const SPACE: u16 = 274;
pub(super) const DIGIT: u16 = 18;
pub(super) const DOT: u16 = 16;
const AHEM: &[u8] =
    include_bytes!("../../../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn u16n(value: usize) -> u16 {
    u16::try_from(value).unwrap()
}
fn longs(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}
fn table(tag: &[u8; 4]) -> &'static [u8] {
    let count = u16::from_be_bytes(AHEM[4..6].try_into().unwrap()) as usize;
    let row = AHEM[12..12 + count * 16]
        .chunks_exact(16)
        .find(|row| &row[..4] == tag)
        .unwrap();
    let start = u32::from_be_bytes(row[8..12].try_into().unwrap()) as usize;
    let len = u32::from_be_bytes(row[12..16].try_into().unwrap()) as usize;
    &AHEM[start..start + len]
}

pub(super) fn layout_table(tag: &[u8; 4], lookups: &[(u16, Vec<u8>)]) -> Vec<u8> {
    // DFLT default LangSys -> selected feature -> every supplied lookup.
    let mut scripts = words(&[1]);
    scripts.extend(*b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0]));
    let mut features = words(&[1]);
    features.extend(tag);
    features.extend(words(&[8, 0, u16n(lookups.len())]));
    features.extend(words(&(0..lookups.len()).map(u16n).collect::<Vec<_>>()));
    let mut list = words(&[u16n(lookups.len())]);
    let mut offset = 2 + 2 * lookups.len();
    for (_, data) in lookups {
        list.extend(words(&[u16n(offset)]));
        offset += 8 + data.len();
    }
    for (kind, data) in lookups {
        list.extend(words(&[*kind, 0, 1, 8]));
        list.extend(data);
    }
    let mut result = words(&[
        1,
        0,
        10,
        u16n(10 + scripts.len()),
        u16n(10 + scripts.len() + features.len()),
    ]);
    result.extend(scripts);
    result.extend(features);
    result.extend(list);
    result
}

pub(super) fn replace() -> (u16, Vec<u8>) {
    (1, words(&[2, 8, 1, R, 1, 1, A]))
}
pub(super) fn ligature() -> (u16, Vec<u8>) {
    (4, words(&[1, 18, 1, 8, 1, 4, R, 2, B, 1, 1, A]))
}
pub(super) fn single() -> (u16, Vec<u8>) {
    // A: x+30,y+40,xAdvance+99 (irrelevant in TTB), yAdvance+50.
    (1, words(&[1, 14, 15, 30, 40, 99, 50, 1, 1, A]))
}
pub(super) fn pair() -> (u16, Vec<u8>) {
    // Pair A,B has both real ValueRecords; coverage follows the PairSet.
    (
        2,
        words(&[
            1,
            32,
            15,
            15,
            1,
            12,
            1,
            B,
            30,
            40,
            99,
            50,
            (-20i16) as u16,
            (-30i16) as u16,
            88,
            (-10i16) as u16,
            1,
            1,
            A,
        ]),
    )
}
pub(super) fn mark(
    base: u16,
    mark: u16,
    base_anchor: (u16, u16),
    mark_anchor: (u16, u16),
    kind: u16,
) -> (u16, Vec<u8>) {
    // MarkBase and MarkMark format1 share these one-class record layouts.
    (
        kind,
        words(&[
            1,
            12,
            18,
            1,
            24,
            36,
            1,
            1,
            mark,
            1,
            1,
            base,
            1,
            0,
            6,
            1,
            mark_anchor.0,
            mark_anchor.1,
            1,
            4,
            1,
            base_anchor.0,
            base_anchor.1,
        ]),
    )
}
pub(super) fn cursive() -> (u16, Vec<u8>) {
    // A exit(300,500), B entry(100,200), no other anchors.
    (
        3,
        words(&[1, 14, 2, 0, 22, 28, 0, 1, 2, A, B, 1, 300, 500, 1, 100, 200]),
    )
}

pub(super) fn morx() -> Vec<u8> {
    // A real all-directions noncontextual format8 lookup maps A -> R.
    let mut data = words(&[2, 0]);
    data.extend(longs(&[1, 1, 36, 0, 1, 20, 0x2000_0004, 1]));
    data.extend(words(&[8, A, 1, R]));
    data
}
pub(super) fn kern() -> Vec<u8> {
    // Windows format0, vertical coverage, one actual A/B pair+25.
    words(&[0, 1, 0, 20, 0, 1, 6, 0, 0, A, B, 25])
}
pub(super) fn trak() -> Vec<u8> {
    // Version1, vertical data at12, one track0/size12/value25.
    let mut data = words(&[1, 0, 0, 0, 12, 0, 1, 1]);
    data.extend(longs(&[28, 0]));
    data.extend(words(&[256, 32]));
    data.extend(longs(&[12 << 16]));
    data.extend(words(&[25]));
    data
}
pub(super) fn stat() -> Vec<u8> {
    // Valid STAT1.0 with empty axis inventory, not an invalid tag sentinel.
    let mut data = words(&[1, 0, 8, 0]);
    data.extend(longs(&[0]));
    data.extend(words(&[0]));
    data.extend(longs(&[0]));
    data
}
fn cmap() -> Vec<u8> {
    let entries = [
        (32, SPACE),
        (46, DOT),
        (48, DIGIT),
        (65, A),
        (66, B),
        (82, R),
        (0x300, MARK),
        (0x315, MARK2),
        (0x25cc, R),
    ];
    let mut format = words(&[12, 0]);
    format.extend(longs(&[
        16 + entries.len() as u32 * 12,
        0,
        entries.len() as u32,
    ]));
    for (scalar, glyph) in entries {
        format.extend(longs(&[scalar, scalar, u32::from(glyph)]));
    }
    let mut result = words(&[0, 1, 3, 10]);
    result.extend(longs(&[12]));
    result.extend(format);
    result
}
fn gdef() -> Vec<u8> {
    // ClassDef format2: actual A/B bases and MARK/MARK2 marks.
    words(&[1, 0, 12, 0, 0, 0, 2, 2, A, B, 1, MARK, MARK2, 3])
}

pub(super) fn with_font(
    native: bool,
    odd: bool,
    gsub: Option<&[u8]>,
    gpos: Option<&[u8]>,
    extra: &[([u8; 4], &[u8])],
    f: impl FnOnce(&Shaper<'_>),
) {
    let cmap = cmap();
    with_cmap_font(native, odd, gsub, gpos, extra, &cmap, f);
}

/// Same owned native records with one caller-authored cmap, never duplicate
/// SFNT directory entries. Existing fixture callers retain the original cmap.
pub(super) fn with_cmap_font(
    native: bool,
    odd: bool,
    gsub: Option<&[u8]>,
    gpos: Option<&[u8]>,
    extra: &[([u8; 4], &[u8])],
    cmap: &[u8],
    f: impl FnOnce(&Shaper<'_>),
) {
    let count = u16::from_be_bytes(table(b"maxp")[4..6].try_into().unwrap());
    let mut head = table(b"head").to_vec();
    head[50..52].copy_from_slice(&1u16.to_be_bytes());
    let mut hhea = table(b"hhea").to_vec();
    hhea[4..6].copy_from_slice(&1000i16.to_be_bytes());
    hhea[6..8].copy_from_slice(&(-200i16).to_be_bytes());
    hhea[34..36].copy_from_slice(&count.to_be_bytes());
    let mut os2 = table(b"OS/2").to_vec();
    os2[68..70].copy_from_slice(&1000i16.to_be_bytes());
    os2[70..72].copy_from_slice(&(-200i16).to_be_bytes());
    let mut vhea = vec![0; 36];
    vhea[..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    vhea[34..36].copy_from_slice(&count.to_be_bytes());
    let mut hmtx = Vec::new();
    let mut vmtx = Vec::new();
    let mut loca = Vec::new();
    let mut glyf = Vec::new();
    for gid in 0..count {
        let (width, height, tsb, ink_width, ink_height) = match gid {
            A => (if odd { 1001 } else { 1000 }, 1200, 100, 1000, 800),
            B => (1000, 1400, 200, 1000, 800),
            R => (1201, 1600, 200, 1000, 800),
            MARK | MARK2 => (200, 400, -50, 100, 200),
            SPACE => (501, 1001, 100, 1000, 800),
            DIGIT => (1000, 1701, 100, 1000, 800),
            DOT => (500, 701, 100, 500, 800),
            0 => (501, 600, -900, 1000, 800),
            _ => (1000, 1200, 100, 1000, 800),
        };
        hmtx.extend(words(&[width, 0]));
        vmtx.extend(words(&[height, tsb as i16 as u16]));
        loca.extend(longs(&[u32::try_from(glyf.len()).unwrap()]));
        // One real four-point rectangle. Long coordinates are raw design units.
        glyf.extend(words(&[1, 0, 0, ink_width, ink_height, 3, 0]));
        glyf.extend([1, 1, 1, 1]);
        glyf.extend(words(&[0, ink_width, 0, (-(ink_width as i16)) as u16]));
        glyf.extend(words(&[0, 0, ink_height, 0]));
    }
    loca.extend(longs(&[u32::try_from(glyf.len()).unwrap()]));
    let gdef = gdef();
    let mut tables = vec![
        (*b"head", head.as_slice()),
        (*b"hhea", hhea.as_slice()),
        (*b"OS/2", os2.as_slice()),
        (*b"hmtx", hmtx.as_slice()),
        (*b"glyf", glyf.as_slice()),
        (*b"loca", loca.as_slice()),
        (*b"cmap", cmap),
        (*b"GDEF", gdef.as_slice()),
    ];
    if native {
        tables.extend([(*b"vhea", vhea.as_slice()), (*b"vmtx", vmtx.as_slice())]);
    }
    if let Some(data) = gsub {
        tables.push((*b"GSUB", data));
    }
    if let Some(data) = gpos {
        tables.push((*b"GPOS", data));
    }
    tables.extend_from_slice(extra);
    with_tables(&tables, f);
}

/// Read the actual authored records from this same configured font, not a
/// glyph-ID answer table or Text's future catalogue/metric-error adapter.
pub(super) fn metrics(font: &Shaper<'_>, gid: u32) -> VerticalNominalMetrics {
    let glyph = GlyphId::new(gid);
    let horizontal = font.font.hmtx().unwrap();
    let width = horizontal.advance(glyph).unwrap();
    if let Ok(vertical) = font.font.vmtx() {
        let height = vertical.advance(glyph).unwrap();
        let tsb = vertical.side_bearing(glyph).unwrap();
        let head = font.font.head().unwrap();
        let loca = font
            .font
            .loca(Some(head.index_to_loc_format() != 0))
            .unwrap();
        let glyf = font.font.glyf().unwrap();
        let ink = loca.get_glyf(glyph, &glyf).unwrap().unwrap();
        VerticalNominalMetrics::new(
            u32::from(height),
            i32::from(width),
            i32::from(ink.y_max()) + i32::from(tsb),
        )
    } else {
        let horizontal = font.font.hhea().unwrap();
        let ascent = i32::from(horizontal.ascender().to_i16());
        let descent = i32::from(horizontal.descender().to_i16());
        VerticalNominalMetrics::new(
            u32::try_from(ascent - descent).unwrap(),
            i32::from(width),
            ascent,
        )
    }
}
