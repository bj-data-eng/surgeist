//! Authored OpenType mark positioning adapted from Layout canonical_fonts.rs
//! and Text test_fonts/shaping.rs. The unchanged base is Render
//! tests/fixtures/fonts/ahem/Ahem.ttf, licensed and identified in that
//! directory PROVENANCE.md (public domain/CC0 fallback). Only cmap/GDEF/GPOS
//! are replaced; real registered Text preparation shapes these font bytes.
//! No glyph rows, advances, or outline geometry are injected into consumers.

const AHEM: &[u8] = super::AHEM;

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + i * 16)
        .find(|r| bytes[*r..*r + 4] == tag)
        .unwrap();
    let offset = u32_at(bytes, record + 8) as usize;
    let len = u32_at(bytes, record + 12) as usize;
    &bytes[offset..offset + len]
}
fn replace_tables(bytes: &[u8], replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let mut tables: Vec<_> = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| {
            let r = 12 + i * 16;
            let tag: [u8; 4] = bytes[r..r + 4].try_into().unwrap();
            (tag, table(bytes, tag).to_vec())
        })
        .collect();
    for (tag, data) in replacements {
        if let Some(t) = tables.iter_mut().find(|t| t.0 == tag) {
            t.1 = data;
        } else {
            tables.push((tag, data));
        }
    }
    tables.sort_by_key(|t| t.0);
    let mut output = vec![0; 12 + tables.len() * 16];
    output[..4].copy_from_slice(&bytes[..4]);
    output[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    let power =
        tables.len().next_power_of_two() / if tables.len().is_power_of_two() { 1 } else { 2 };
    output[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    output[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    output[10..12].copy_from_slice(
        &u16::try_from((tables.len() - power) * 16)
            .unwrap()
            .to_be_bytes(),
    );
    for (i, (tag, mut data)) in tables.into_iter().enumerate() {
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        let r = 12 + i * 16;
        let offset = u32::try_from(output.len()).unwrap();
        output[r..r + 4].copy_from_slice(&tag);
        output[r + 4..r + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        output[r + 8..r + 12].copy_from_slice(&offset.to_be_bytes());
        output[r + 12..r + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    while !output.len().is_multiple_of(4) {
        output.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&output));
    let head = (0..usize::from(u16_at(&output, 4)))
        .map(|index| 12 + index * 16)
        .find(|record| output[*record..*record + 4] == *b"head")
        .expect("Ahem has a real head table");
    let head_offset = usize::try_from(u32_at(&output, head + 8)).unwrap();
    output[head_offset + 8..head_offset + 12].copy_from_slice(&adjustment.to_be_bytes());
    output
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn offset(value: usize) -> u16 {
    u16::try_from(value).expect("small authored OpenType table fits Offset16")
}

fn layout_table(script: [u8; 4], mut features: Vec<([u8; 4], u16, Vec<u8>)>) -> Vec<u8> {
    features.sort_by_key(|feature| feature.0);
    let count = offset(features.len());
    let mut scripts = words(&[1]);
    scripts.extend(script);
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, count]));
    for index in 0..count {
        scripts.extend(words(&[index]));
    }
    let mut feature_list = words(&[count]);
    for (index, feature) in features.iter().enumerate() {
        feature_list.extend(feature.0);
        feature_list.extend(words(&[offset(2 + 6 * features.len() + index * 6)]));
    }
    for index in 0..count {
        feature_list.extend(words(&[0, 1, index]));
    }
    let mut lookup_list = words(&[count]);
    lookup_list.resize(2 + 2 * features.len(), 0);
    for (index, (_, kind, subtable)) in features.iter().enumerate() {
        let start = offset(lookup_list.len());
        lookup_list[2 + index * 2..4 + index * 2].copy_from_slice(&start.to_be_bytes());
        lookup_list.extend(words(&[*kind, 0, 1, 8]));
        lookup_list.extend(subtable);
    }
    let mut output = words(&[
        1,
        0,
        10,
        offset(10 + scripts.len()),
        offset(10 + scripts.len() + feature_list.len()),
    ]);
    output.extend(scripts);
    output.extend(feature_list);
    output.extend(lookup_list);
    output
}

fn cmap(mappings: &[(u32, u32)]) -> Vec<u8> {
    let mut mappings = mappings.to_vec();
    mappings.sort_unstable();
    let mut output = words(&[0, 1, 3, 10]);
    output.extend(12_u32.to_be_bytes());
    output.extend(words(&[12, 0]));
    output.extend(
        u32::try_from(16 + mappings.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    output.extend(0_u32.to_be_bytes());
    output.extend(u32::try_from(mappings.len()).unwrap().to_be_bytes());
    for (codepoint, glyph) in mappings {
        output.extend(codepoint.to_be_bytes());
        output.extend(codepoint.to_be_bytes());
        output.extend(glyph.to_be_bytes());
    }
    output
}

/// GPOS mark-to-base: base anchor(400,600), mark anchor(100,200).
/// At size 10/UPEM 1000, the actual mark is at logical (3,-4), advance 0.
pub(super) fn mark_ahem() -> Vec<u8> {
    let mappings = [
        (0x41, 35),
        (0x42, 36),
        (0x58, 58),
        (0x300, 100),
        (0x301, 100),
        (0x315, 100),
    ];
    let gdef = words(&[1, 0, 12, 0, 0, 0, 2, 2, 35, 35, 1, 100, 100, 3]);
    let subtable = words(&[
        1, 34, 40, 1, 12, 24, 1, 0, 6, 1, 100, 200, 1, 4, 1, 400, 600, 1, 1, 100, 1, 1, 35,
    ]);
    replace_tables(
        AHEM,
        vec![
            (*b"cmap", cmap(&mappings)),
            (*b"GDEF", gdef),
            (
                *b"GPOS",
                layout_table(*b"DFLT", vec![(*b"mark", 4, subtable)]),
            ),
        ],
    )
}
