//! Repository-authored static font: a framed four-digit hexadecimal depiction.
//! All coordinates, contours, cmap, metrics and checksums are actual font data.

use linebender_resource_handle::Blob;
use skrifa::{MetadataProvider, raw::TableProvider};

use crate::{Error, ErrorCode, Result};

const EM: u16 = 1000;
const ADVANCE: u16 = 900;
const ASCENT: i16 = 800;
const DESCENT: i16 = -200;
const HEX: [[u8; 5]; 16] = [
    [7, 5, 5, 5, 7],
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
    [7, 5, 7, 1, 7],
    [7, 5, 7, 5, 5],
    [6, 5, 6, 5, 6],
    [7, 4, 4, 4, 7],
    [6, 5, 5, 5, 6],
    [7, 4, 6, 4, 7],
    [7, 4, 6, 4, 4],
];

fn u16_be(output: &mut Vec<u8>, value: u16) {
    output.extend(value.to_be_bytes());
}
fn i16_be(output: &mut Vec<u8>, value: i16) {
    output.extend(value.to_be_bytes());
}
fn u32_be(output: &mut Vec<u8>, value: u32) {
    output.extend(value.to_be_bytes());
}

fn rectangle(points: &mut Vec<(i16, i16)>, x: i16, y: i16, w: i16, h: i16) {
    points.extend([(x, y), (x + w, y), (x + w, y + h), (x, y + h)]);
}

fn glyph(code_point: u32) -> (Vec<u8>, u16, u16) {
    let mut points = Vec::new();
    for (x, y, width, height) in [
        (40, 0, 40, 800),
        (820, 0, 40, 800),
        (80, 0, 740, 40),
        (80, 760, 740, 40),
    ] {
        rectangle(&mut points, x, y, width, height);
    }
    for column in 0..4 {
        let digit = ((code_point >> ((3 - column) * 4)) & 15) as usize;
        for (row, &bits) in HEX[digit].iter().enumerate() {
            for pixel in 0..3 {
                if bits & (1 << (2 - pixel)) != 0 {
                    rectangle(
                        &mut points,
                        110 + column as i16 * 180 + pixel * 40,
                        460 - row as i16 * 40,
                        32,
                        32,
                    );
                }
            }
        }
    }
    let contours = u16::try_from(points.len() / 4).expect("fixed glyph contours fit u16");
    let count = u16::try_from(points.len()).expect("fixed glyph points fit u16");
    let mut output = Vec::new();
    i16_be(
        &mut output,
        i16::try_from(contours).expect("fixed contours fit i16"),
    );
    for value in [40, 0, 860, 800] {
        i16_be(&mut output, value);
    }
    for contour in 0..contours {
        u16_be(&mut output, contour * 4 + 3);
    }
    u16_be(&mut output, 0); // no instructions
    output.extend(std::iter::repeat_n(1, points.len())); // on-curve, full signed deltas
    let mut previous = 0;
    for &(x, _) in &points {
        i16_be(&mut output, x - previous);
        previous = x;
    }
    previous = 0;
    for &(_, y) in &points {
        i16_be(&mut output, y - previous);
        previous = y;
    }
    (output, count, contours)
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

pub(crate) fn make() -> Result<Blob<u8>> {
    let values: Vec<_> = (0..=159).filter(|&cp| super::is_visible(cp)).collect();
    let count = u16::try_from(values.len() + 1).expect("the closed Cc repertoire fits u16");
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    let mut max_points = 0;
    let mut max_contours = 0;
    for cp in std::iter::once(0).chain(values.iter().copied()) {
        u32_be(
            &mut loca,
            u32::try_from(glyf.len()).expect("fixed glyph font fits u32"),
        );
        let (data, points, contours) = glyph(cp);
        max_points = max_points.max(points);
        max_contours = max_contours.max(contours);
        glyf.extend(data);
        if !glyf.len().is_multiple_of(2) {
            glyf.push(0);
        }
    }
    u32_be(
        &mut loca,
        u32::try_from(glyf.len()).expect("fixed glyph font fits u32"),
    );
    let mut cmap = Vec::new();
    for n in [0, 1, 3, 10] {
        u16_be(&mut cmap, n);
    }
    u32_be(&mut cmap, 12);
    u16_be(&mut cmap, 12);
    u16_be(&mut cmap, 0);
    u32_be(&mut cmap, 16 + values.len() as u32 * 12);
    u32_be(&mut cmap, 0);
    u32_be(&mut cmap, values.len() as u32);
    for (index, &cp) in values.iter().enumerate() {
        u32_be(&mut cmap, cp);
        u32_be(&mut cmap, cp);
        u32_be(&mut cmap, index as u32 + 1);
    }
    let mut head = Vec::new();
    for n in [0x0001_0000, 0x0001_0000, 0, 0x5f0f_3cf5] {
        u32_be(&mut head, n);
    }
    u16_be(&mut head, 0);
    u16_be(&mut head, EM);
    head.extend([0; 16]);
    for n in [40, 0, 860, 800] {
        i16_be(&mut head, n);
    }
    for n in [0, 8, 2, 1, 0] {
        i16_be(&mut head, n);
    }
    let mut hhea = Vec::new();
    u32_be(&mut hhea, 0x0001_0000);
    for n in [
        ASCENT,
        DESCENT,
        0,
        ADVANCE as i16,
        40,
        40,
        860,
        1,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ] {
        i16_be(&mut hhea, n);
    }
    u16_be(&mut hhea, count);
    let mut hmtx = Vec::new();
    for _ in 0..count {
        u16_be(&mut hmtx, ADVANCE);
        i16_be(&mut hmtx, 40);
    }
    let mut maxp = Vec::new();
    u32_be(&mut maxp, 0x0001_0000);
    u16_be(&mut maxp, count);
    for n in [max_points, max_contours, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0] {
        u16_be(&mut maxp, n);
    }
    let mut os2 = vec![0; 78];
    for (at, n) in [
        (2, ADVANCE),
        (4, 400),
        (6, 5),
        (26, 50),
        (28, 300),
        (62, 0x40),
        (64, 0),
        (66, 159),
        (68, ASCENT as u16),
        (70, DESCENT as u16),
        (74, ASCENT as u16),
        (76, (-DESCENT) as u16),
    ] {
        os2[at..at + 2].copy_from_slice(&n.to_be_bytes());
    }
    os2[58..62].copy_from_slice(b"SURG");
    let mut post = Vec::new();
    u32_be(&mut post, 0x0003_0000);
    u32_be(&mut post, 0);
    i16_be(&mut post, -100);
    i16_be(&mut post, 50);
    u32_be(&mut post, 1);
    post.extend([0; 16]);
    let labels = [
        (1, "Surgeist Control"),
        (2, "Regular"),
        (4, "Surgeist Control Regular"),
        (6, "SurgeistControl-Regular"),
    ];
    let mut name = Vec::new();
    let mut strings = Vec::new();
    u16_be(&mut name, 0);
    u16_be(&mut name, labels.len() as u16);
    u16_be(&mut name, 6 + labels.len() as u16 * 12);
    for (id, label) in labels {
        let data: Vec<_> = label.encode_utf16().flat_map(u16::to_be_bytes).collect();
        for n in [3, 1, 0x409, id, data.len() as u16, strings.len() as u16] {
            u16_be(&mut name, n);
        }
        strings.extend(data);
    }
    name.extend(strings);
    let mut tables = vec![
        (*b"OS/2", os2),
        (*b"cmap", cmap),
        (*b"glyf", glyf),
        (*b"head", head),
        (*b"hhea", hhea),
        (*b"hmtx", hmtx),
        (*b"loca", loca),
        (*b"maxp", maxp),
        (*b"name", name),
        (*b"post", post),
    ];
    tables.sort_unstable_by_key(|(tag, _)| *tag);
    let mut bytes = Vec::new();
    u32_be(&mut bytes, 0x0001_0000);
    u16_be(&mut bytes, tables.len() as u16);
    for n in [128, 3, 32] {
        u16_be(&mut bytes, n);
    }
    bytes.resize(12 + tables.len() * 16, 0);
    let mut head_offset = 0;
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let offset = bytes.len();
        let record = 12 + i * 16;
        bytes[record..record + 4].copy_from_slice(&tag);
        bytes[record + 4..record + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        bytes[record + 8..record + 12].copy_from_slice(&(offset as u32).to_be_bytes());
        bytes[record + 12..record + 16].copy_from_slice(&(data.len() as u32).to_be_bytes());
        if tag == *b"head" {
            head_offset = offset;
        }
        bytes.extend(data);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    bytes[head_offset + 8..head_offset + 12].copy_from_slice(&adjustment.to_be_bytes());
    let face = skrifa::FontRef::new(&bytes).map_err(|_| invalid())?;
    face.head().map_err(|_| invalid())?;
    face.hhea().map_err(|_| invalid())?;
    face.hmtx().map_err(|_| invalid())?;
    face.maxp().map_err(|_| invalid())?;
    face.cmap().map_err(|_| invalid())?;
    Ok(Blob::from(bytes))
}

fn invalid() -> Error {
    Error::new(
        ErrorCode::FontLoadFailed,
        "intrinsic control font is invalid",
    )
}

/// Admit the complete authored resource through its actual production readers
/// before Catalogue retains it. A partial directory parse is insufficient.
pub(crate) fn validate(font: &crate::FontData) -> Result<()> {
    let instance = crate::EffectiveFontInstance::qualify_synthesized(
        font.face(),
        f32::from(EM),
        &[],
        crate::FontSynthesis::default(),
    )
    .map_err(|_| invalid())?;
    instance
        .metrics(font.catalogue_identity())
        .map_err(|_| invalid())?;
    let face = skrifa::FontRef::from_index(font.bytes(), font.index()).map_err(|_| invalid())?;
    let charmap = face.charmap();
    for (index, cp) in (0..=159).filter(|&cp| super::is_visible(cp)).enumerate() {
        let glyph = charmap.map(cp).ok_or_else(invalid)?;
        if glyph.to_u32() != index as u32 + 1 {
            return Err(invalid());
        }
        let bounds = instance
            .glyph_outline(glyph.to_u32(), font.catalogue_identity())
            .map_err(|_| invalid())?
            .bounds()
            .ok_or_else(invalid)?;
        if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
            return Err(invalid());
        }
    }
    Ok(())
}
