//! Real table operands for glyph-free used-upright observations.
use crate::{baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical};

pub(super) fn native(version: u32, over: i16, under: i16, gap: i16) -> Vec<u8> {
    let font = vertical::native();
    let mut header = sfnt::table(&font, *b"vhea").to_vec();
    header[..4].copy_from_slice(&version.to_be_bytes());
    for (at, value) in [(4, over), (6, under), (8, gap)] {
        header[at..at + 2].copy_from_slice(&value.to_be_bytes());
    }
    sfnt::replace_tables(&font, vec![(*b"vhea", header)])
}

pub(super) fn absolute(font: &[u8]) -> Vec<u8> {
    sfnt::replace_tables(
        font,
        vec![(
            *b"BASE",
            base::base(
                0,
                None,
                Some(base::single_axis(&[*b"ideo", *b"idtp"], &[0, 1200])),
                None,
            ),
        )],
    )
}

pub(super) fn fractional_vertical() -> Vec<u8> {
    let axis = base::Axis {
        tags: vec![*b"ideo", *b"idtp"],
        scripts: vec![base::Script {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![
                base::Coord::Design(0),
                base::Coord::Adjusted {
                    coordinate: 1000,
                    adjustment: Some(base::variation_index(0, 0)),
                },
            ],
        }],
    };
    sfnt::replace_tables(
        &base::two_axis_ahem(),
        vec![(
            *b"BASE",
            base::base(
                1,
                None,
                Some(axis),
                Some(base::region_store(
                    &[[0, 16384, 16384], [0, 16384, 16384]],
                    3,
                )),
            ),
        )],
    )
}

pub(super) fn mvar(records: &[([u8; 4], i16)]) -> Vec<u8> {
    assert!(records.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let mut bytes = Vec::new();
    let count = u16::try_from(records.len()).unwrap();
    for word in [
        1,
        0,
        0,
        8,
        count,
        if count == 0 { 0 } else { 12 + 8 * count },
    ] {
        bytes.extend(word.to_be_bytes());
    }
    if count == 0 {
        return bytes;
    }
    for (row, &(tag, _)) in records.iter().enumerate() {
        bytes.extend(tag);
        bytes.extend(0u16.to_be_bytes());
        bytes.extend(u16::try_from(row).unwrap().to_be_bytes());
    }
    // Real ItemVariationStore1: one axis, one [0,1,1] region, N signed rows.
    bytes.extend(1u16.to_be_bytes());
    bytes.extend(12u32.to_be_bytes());
    bytes.extend(1u16.to_be_bytes());
    bytes.extend(22u32.to_be_bytes());
    for word in [1i16, 1, 0, 16384, 16384] {
        bytes.extend(word.to_be_bytes());
    }
    for word in [count, 1, 1, 0] {
        bytes.extend(word.to_be_bytes());
    }
    for (_, delta) in records {
        bytes.extend(delta.to_be_bytes());
    }
    bytes
}

pub(super) fn variable(records: &[([u8; 4], i16)]) -> Vec<u8> {
    let font = sfnt::variable_ahem();
    let static_vertical = native(0x0001_1000, 700, -300, 42);
    sfnt::replace_tables(
        &font,
        vec![
            (*b"vhea", sfnt::table(&static_vertical, *b"vhea").to_vec()),
            (*b"vmtx", sfnt::table(&static_vertical, *b"vmtx").to_vec()),
            (*b"MVAR", mvar(records)),
        ],
    )
}

pub(super) fn odd_em(font: &[u8]) -> Vec<u8> {
    let mut head = sfnt::table(font, *b"head").to_vec();
    head[18..20].copy_from_slice(&1001u16.to_be_bytes());
    sfnt::replace_tables(font, vec![(*b"head", head)])
}
