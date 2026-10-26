//! Real Ahem derivative with explicit blank separators and a zero-advance mark.
//! Licensed glyf/loca stay unchanged; see ../fixtures/fonts/ahem/PROVENANCE.md.

use std::sync::OnceLock;

const AHEM: &[u8] = include_bytes!("../fixtures/fonts/ahem/Ahem.ttf");

fn read16(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn read32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn build() -> Vec<u8> {
    let mappings = [
        (0x20_u32, 3_u32),
        (0x41, 35),
        (0x42, 36),
        (0x43, 37),
        (0x58, 58),
        (0x61, 67),
        (0x62, 68),
        (0xa0, 3),
        (0x301, 100),
        (0x1680, 3),
        (0x2002, 3),
        (0x2003, 3),
        (0x2007, 3),
        (0x202f, 3),
        (0x3000, 1),
        (0xff21, 277),
        (0xff22, 274),
    ];
    let mut cmap = [0_u16, 1, 3, 10]
        .into_iter()
        .flat_map(u16::to_be_bytes)
        .collect::<Vec<_>>();
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend([12_u16, 0].into_iter().flat_map(u16::to_be_bytes));
    cmap.extend(
        u32::try_from(16 + mappings.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(u32::try_from(mappings.len()).unwrap().to_be_bytes());
    for (scalar, glyph) in mappings {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let mut tables = (0..usize::from(read16(AHEM, 4)))
        .map(|index| {
            let at = 12 + index * 16;
            let tag: [u8; 4] = AHEM[at..at + 4].try_into().unwrap();
            let start = usize::try_from(read32(AHEM, at + 8)).unwrap();
            let length = usize::try_from(read32(AHEM, at + 12)).unwrap();
            let mut data = AHEM[start..start + length].to_vec();
            if tag == *b"cmap" {
                data = cmap.clone();
            }
            if tag == *b"head" {
                assert_eq!(read16(&data, 18), 1000);
                data[8..12].fill(0);
            }
            if tag == *b"hmtx" {
                for (glyph, advance) in [
                    (1, 1000_u16),
                    (3, 1000),
                    (35, 1000),
                    (36, 1000),
                    (37, 1000),
                    (58, 1000),
                    (67, 1000),
                    (68, 1000),
                    (100, 0),
                    (274, 1000),
                    (277, 1000),
                ] {
                    data[glyph * 4..glyph * 4 + 2].copy_from_slice(&advance.to_be_bytes());
                }
            }
            (tag, data)
        })
        .collect::<Vec<_>>();
    tables.sort_by_key(|entry| entry.0);
    let count = tables.len();
    let mut bytes = vec![0; 12 + count * 16];
    bytes[..4].copy_from_slice(&AHEM[..4]);
    bytes[4..6].copy_from_slice(&u16::try_from(count).unwrap().to_be_bytes());
    let power = count.next_power_of_two() / if count.is_power_of_two() { 1 } else { 2 };
    bytes[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    bytes[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    bytes[10..12].copy_from_slice(&u16::try_from((count - power) * 16).unwrap().to_be_bytes());
    for (index, (tag, data)) in tables.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let record = 12 + index * 16;
        let start = u32::try_from(bytes.len()).unwrap();
        bytes[record..record + 4].copy_from_slice(&tag);
        bytes[record + 4..record + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        bytes[record + 8..record + 12].copy_from_slice(&start.to_be_bytes());
        bytes[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        bytes.extend(data);
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    let record = (0..count)
        .map(|index| 12 + index * 16)
        .find(|at| bytes[*at..*at + 4] == *b"head")
        .unwrap();
    let start = usize::try_from(read32(&bytes, record + 8)).unwrap();
    bytes[start + 8..start + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    bytes
}

pub fn bytes() -> &'static [u8] {
    static FONT: OnceLock<Vec<u8>> = OnceLock::new();
    FONT.get_or_init(build)
}
