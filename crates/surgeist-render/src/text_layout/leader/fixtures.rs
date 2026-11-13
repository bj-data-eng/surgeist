//! Authored Ahem derivatives reuse existing Render fixture table layouts.
use super::AHEM;
fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
pub(super) fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + i * 16)
        .find(|&record| bytes[record..record + 4] == tag)
        .unwrap();
    let offset = u32_at(bytes, record + 8) as usize;
    let len = u32_at(bytes, record + 12) as usize;
    &bytes[offset..offset + len]
}
pub(super) fn replace_tables(replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let mut tables: Vec<_> = (0..usize::from(u16_at(AHEM, 4)))
        .map(|i| {
            let tag: [u8; 4] = AHEM[12 + i * 16..16 + i * 16].try_into().unwrap();
            (tag, table(AHEM, tag).to_vec())
        })
        .collect();
    for (tag, data) in replacements {
        if let Some(old) = tables.iter_mut().find(|old| old.0 == tag) {
            old.1 = data;
        } else {
            tables.push((tag, data));
        }
    }
    tables.sort_by_key(|entry| entry.0);
    let mut result = vec![0; 12 + tables.len() * 16];
    result[..4].copy_from_slice(&AHEM[..4]);
    result[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    let power =
        tables.len().next_power_of_two() / if tables.len().is_power_of_two() { 1 } else { 2 };
    result[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    result[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    result[10..12].copy_from_slice(
        &u16::try_from((tables.len() - power) * 16)
            .unwrap()
            .to_be_bytes(),
    );
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        while !result.len().is_multiple_of(4) {
            result.push(0);
        }
        let record = 12 + i * 16;
        let offset = u32::try_from(result.len()).unwrap();
        result[record..record + 4].copy_from_slice(&tag);
        result[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
        result[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        result.extend(data);
    }
    result
}
pub(super) fn composite_x_ahem(scale: Option<i16>) -> Vec<u8> {
    let mut composite = Vec::new();
    for word in [
        -1_i16,
        0,
        -200,
        1000,
        800,
        if scale.is_some() { 11 } else { 3 },
        35,
        0,
        0,
    ] {
        composite.extend(word.to_be_bytes());
    }
    if let Some(scale) = scale {
        composite.extend(scale.to_be_bytes());
    }
    let loca = table(AHEM, *b"loca");
    let glyf = table(AHEM, *b"glyf");
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let (mut new_loca, mut new_glyf) = (Vec::new(), Vec::new());
    for id in 0..count {
        new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
        if id == 58 {
            new_glyf.extend_from_slice(&composite);
        } else {
            new_glyf.extend_from_slice(
                &glyf[usize::from(u16_at(loca, id * 2)) * 2
                    ..usize::from(u16_at(loca, (id + 1) * 2)) * 2],
            );
        }
        if !new_glyf.len().is_multiple_of(2) {
            new_glyf.push(0);
        }
    }
    new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
    replace_tables(vec![(*b"loca", new_loca), (*b"glyf", new_glyf)])
}

pub(super) fn zero_advance() -> Vec<u8> {
    let mut hmtx = table(AHEM, *b"hmtx").to_vec();
    hmtx[35 * 4..35 * 4 + 2].fill(0);
    replace_tables(vec![(*b"hmtx", hmtx)])
}

pub(super) fn empty_color() -> Vec<u8> {
    color(0)
}

pub(super) fn color(alpha: u8) -> Vec<u8> {
    let mut colr = Vec::new();
    for n in [0_u16, 1] {
        colr.extend(n.to_be_bytes());
    }
    for n in [14_u32, 20] {
        colr.extend(n.to_be_bytes());
    }
    for n in [1_u16, 58, 0, 1, 58, 0] {
        colr.extend(n.to_be_bytes());
    }
    let mut cpal = Vec::new();
    for n in [0_u16, 1, 1, 1] {
        cpal.extend(n.to_be_bytes());
    }
    cpal.extend(14_u32.to_be_bytes());
    cpal.extend(0_u16.to_be_bytes());
    cpal.extend([0, 0, 255, alpha]);
    replace_tables(vec![(*b"COLR", colr), (*b"CPAL", cpal)])
}

pub(super) fn negative_bearing() -> Vec<u8> {
    let base = composite_x_ahem(None);
    let mut glyf = table(&base, *b"glyf").to_vec();
    let at = usize::from(u16_at(table(&base, *b"loca"), 58 * 2)) * 2;
    glyf[at + 2..at + 4].copy_from_slice(&(-100_i16).to_be_bytes());
    glyf[at + 6..at + 8].copy_from_slice(&900_i16.to_be_bytes());
    glyf[at + 14..at + 16].copy_from_slice(&(-100_i16).to_be_bytes());
    let mut hmtx = table(&base, *b"hmtx").to_vec();
    hmtx[58 * 4 + 2..58 * 4 + 4].copy_from_slice(&(-100_i16).to_be_bytes());
    replace_tables(vec![
        (*b"glyf", glyf),
        (*b"loca", table(&base, *b"loca").to_vec()),
        (*b"hmtx", hmtx),
    ])
}

pub(super) fn fallback_fonts() -> (Vec<u8>, Vec<u8>) {
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12];
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    for n in [28_u32, 0, 1, 65, 65, 35] {
        cmap.extend(n.to_be_bytes());
    }
    let primary = replace_tables(vec![(*b"cmap", cmap)]);
    let mut name = table(AHEM, *b"name").to_vec();
    for index in 0..=name.len() - 8 {
        if name[index..index + 8] == [0, b'A', 0, b'h', 0, b'e', 0, b'm'] {
            name[index + 1] = b'B';
        }
    }
    for index in 0..=name.len() - 4 {
        if &name[index..index + 4] == b"Ahem" {
            name[index] = b'B';
        }
    }
    (primary, replace_tables(vec![(*b"name", name)]))
}

pub(super) fn bitmap(alpha: u8) -> Vec<u8> {
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, 2, 3);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_image_data(&[255, 0, 0, alpha].repeat(6))
        .unwrap();
    drop(writer);
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let start = 4 + (count + 1) * 4;
    let end = start + 8 + png.len();
    let mut sbix = vec![0, 1, 0, 1];
    sbix.extend(1_u32.to_be_bytes());
    sbix.extend(12_u32.to_be_bytes());
    sbix.extend(20_u16.to_be_bytes());
    sbix.extend(72_u16.to_be_bytes());
    for glyph in 0..=count {
        sbix.extend(
            u32::try_from(if glyph <= 58 { start } else { end })
                .unwrap()
                .to_be_bytes(),
        );
    }
    sbix.extend(1_u16.to_be_bytes());
    sbix.extend(2_u16.to_be_bytes());
    sbix.extend(*b"png ");
    sbix.extend(png);
    replace_tables(vec![(*b"sbix", sbix)])
}

pub(super) fn variable_ahem() -> Vec<u8> {
    let mut fvar = Vec::new();
    fvar.extend(0x00010000_u32.to_be_bytes());
    for word in [16_u16, 2, 1, 20, 0, 8] {
        fvar.extend(word.to_be_bytes());
    }
    fvar.extend(*b"wght");
    for word in [0_i32, 0, 65536] {
        fvar.extend(word.to_be_bytes());
    }
    fvar.extend(0_u16.to_be_bytes());
    fvar.extend(256_u16.to_be_bytes());
    let tuple = [
        0, 1, 0, 10, 0, 11, 0xA0, 0, 0x40, 0, 0, 7, 0, 100, 100, 0, 0, 0, 0, 0, 0x87,
    ];
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let offset = u32::try_from(20 + (count + 1) * 4).unwrap();
    let mut gvar = Vec::new();
    gvar.extend(0x00010000_u32.to_be_bytes());
    for word in [1_u16, 0] {
        gvar.extend(word.to_be_bytes());
    }
    gvar.extend(offset.to_be_bytes());
    gvar.extend(u16::try_from(count).unwrap().to_be_bytes());
    gvar.extend(1_u16.to_be_bytes());
    gvar.extend(offset.to_be_bytes());
    for glyph in 0..=count {
        gvar.extend(
            if glyph <= 58 {
                0_u32
            } else {
                u32::try_from(tuple.len()).unwrap()
            }
            .to_be_bytes(),
        );
    }
    gvar.extend(tuple);
    replace_tables(vec![(*b"fvar", fvar), (*b"gvar", gvar)])
}
