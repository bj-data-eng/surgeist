// Repository-owned fixtures for the maintained provider completion contract.
// Ahem is borrowed from the existing licensed Text fixture; no font copy.
use super::buffer::hb_buffer_t;
use crate::{Direction, FontRef, Shaper, ShaperData, UnicodeBuffer};
use alloc::vec::Vec;

const AHEM: &[u8] =
    include_bytes!("../../../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

pub(super) fn with_font(f: impl FnOnce(&Shaper<'_>)) {
    with_bytes(AHEM, f);
}

fn with_bytes(bytes: &[u8], f: impl FnOnce(&Shaper<'_>)) {
    let font = FontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    f(&shaper);
}

pub(super) fn glyphs(count: usize) -> hb_buffer_t {
    let mut input = UnicodeBuffer::new();
    for i in 0..count {
        input.add('A', u32::try_from(i).unwrap());
    }
    input.set_direction(Direction::LeftToRight);
    let mut buffer = input.0;
    // Internal layout routines consume glyph IDs rather than Unicode input.
    for info in &mut buffer.info[..buffer.len] {
        info.glyph_id = 1;
        info.mask = 1;
    }
    buffer
}

pub(super) fn words(words: &[u16]) -> Vec<u8> {
    words.iter().flat_map(|v| v.to_be_bytes()).collect()
}

// One class row: end-of-text advances; every real glyph uses the second entry.
// Both entries have real encoded payload bytes supplied by the caller.
pub(super) fn extended_machine(entry: &[u8], end_entry: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = [4u32, 16, 26, 34]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
    bytes.extend(words(&[8, 0, 2, 1, 1])); // lookup8: glyph0/1 -> class1
    bytes.extend(words(&[0, 1, 1, 1]));
    bytes.extend(end_entry);
    bytes.extend(entry);
    bytes
}

// Minimal original Ahem SFNT plus a real GSUB table. The fixture deliberately
// rebuilds the directory, preserving every original table's actual bytes.
pub(super) fn with_gsub(gsub: &[u8], f: impl FnOnce(&Shaper<'_>)) {
    with_tables(&[(*b"GSUB", gsub)], f);
}

pub(super) fn with_tables(replacements: &[([u8; 4], &[u8])], f: impl FnOnce(&Shaper<'_>)) {
    let count = u16::from_be_bytes([AHEM[4], AHEM[5]]) as usize;
    let mut tables = Vec::new();
    for record in AHEM[12..12 + count * 16].chunks_exact(16) {
        let tag: [u8; 4] = record[..4].try_into().unwrap();
        let start = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
        let len = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
        if !replacements
            .iter()
            .any(|(replacement, _)| *replacement == tag)
        {
            tables.push((tag, &AHEM[start..start + len]));
        }
    }
    tables.extend(replacements.iter().copied());
    tables.sort_by_key(|(tag, _)| *tag);
    let mut font = alloc::vec![0; 12 + tables.len() * 16];
    font[..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    font[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        let start = font.len();
        let record = 12 + i * 16;
        font[record..record + 4].copy_from_slice(&tag);
        font[record + 8..record + 12].copy_from_slice(&u32::try_from(start).unwrap().to_be_bytes());
        font[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        font.extend_from_slice(data);
        while font.len() % 4 != 0 {
            font.push(0);
        }
    }
    with_bytes(&font, f);
}

pub(super) fn gsub_lookup(kind: u16, subtable: &[u8]) -> Vec<u8> {
    // DFLT default language -> ccmp feature -> lookup0.
    let mut gsub = words(&[1, 0, 10, 30, 44, 1]);
    gsub.extend_from_slice(b"DFLT");
    gsub.extend(words(&[8, 4, 0, 0, 0xFFFF, 1, 0]));
    gsub.extend(words(&[1]));
    gsub.extend_from_slice(b"ccmp");
    gsub.extend(words(&[8, 0, 1, 0, 1, 4, kind, 0, 1, 8]));
    gsub.extend_from_slice(subtable);
    gsub
}
