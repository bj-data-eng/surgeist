//! Independently authored BASE inputs on the licensed real Ahem outlines.
//! These helpers encode font bytes; they do not resolve named baselines.
use crate::test_fonts::{self, replace_tables, table, u16_at};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

#[derive(Clone, Debug)]
pub(crate) enum Coord {
    Design(i16),
    Point {
        coordinate: i16,
        glyph: u16,
        point: u16,
    },
    Adjusted {
        coordinate: i16,
        adjustment: Option<Vec<u8>>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct Script {
    pub(crate) tag: [u8; 4],
    pub(crate) default_index: u16,
    pub(crate) coordinates: Vec<Coord>,
}

#[derive(Clone, Debug)]
pub(crate) struct Axis {
    pub(crate) tags: Vec<[u8; 4]>,
    pub(crate) scripts: Vec<Script>,
}

fn u16_bytes(value: usize) -> [u8; 2] {
    u16::try_from(value).unwrap().to_be_bytes()
}

fn coordinate(value: &Coord) -> Vec<u8> {
    let mut bytes = Vec::new();
    match value {
        Coord::Design(value) => {
            bytes.extend(1u16.to_be_bytes());
            bytes.extend(value.to_be_bytes());
        }
        Coord::Point {
            coordinate,
            glyph,
            point,
        } => {
            bytes.extend(2u16.to_be_bytes());
            bytes.extend(coordinate.to_be_bytes());
            bytes.extend(glyph.to_be_bytes());
            bytes.extend(point.to_be_bytes());
        }
        Coord::Adjusted {
            coordinate,
            adjustment,
        } => {
            bytes.extend(3u16.to_be_bytes());
            bytes.extend(coordinate.to_be_bytes());
            let offset = if adjustment.is_some() { 6u16 } else { 0u16 };
            bytes.extend(offset.to_be_bytes());
            if let Some(adjustment) = adjustment {
                bytes.extend(adjustment);
            }
        }
    }
    bytes
}

fn axis(axis: &Axis) -> Vec<u8> {
    let mut tags = Vec::from(u16_bytes(axis.tags.len()));
    for tag in &axis.tags {
        tags.extend(tag);
    }
    let mut scripts = Vec::from(u16_bytes(axis.scripts.len()));
    scripts.resize(2 + 6 * axis.scripts.len(), 0);
    for (index, script) in axis.scripts.iter().enumerate() {
        let record = 2 + 6 * index;
        scripts[record..record + 4].copy_from_slice(&script.tag);
        let offset = u16_bytes(scripts.len());
        scripts[record + 4..record + 6].copy_from_slice(&offset);
        // BaseScript: BaseValues at6, no MinMax or language overrides.
        scripts.extend(6u16.to_be_bytes());
        scripts.extend(0u16.to_be_bytes());
        scripts.extend(0u16.to_be_bytes());
        let mut values = Vec::from(script.default_index.to_be_bytes());
        values.extend(u16_bytes(script.coordinates.len()));
        values.resize(4 + 2 * script.coordinates.len(), 0);
        for (index, value) in script.coordinates.iter().enumerate() {
            let offset = u16_bytes(values.len());
            values[4 + 2 * index..6 + 2 * index].copy_from_slice(&offset);
            values.extend(coordinate(value));
        }
        scripts.extend(values);
    }
    let mut bytes = Vec::from(4u16.to_be_bytes());
    bytes.extend(u16_bytes(4 + tags.len()));
    bytes.extend(tags);
    bytes.extend(scripts);
    bytes
}

pub(crate) fn base(
    minor: u16,
    horizontal: Option<Axis>,
    vertical: Option<Axis>,
    store: Option<Vec<u8>>,
) -> Vec<u8> {
    assert!(minor <= 1);
    assert!(minor == 1 || store.is_none());
    let mut bytes = Vec::from(1u16.to_be_bytes());
    bytes.extend(minor.to_be_bytes());
    bytes.resize(if minor == 1 { 12 } else { 8 }, 0);
    for (field, value) in [(4, horizontal), (6, vertical)] {
        if let Some(value) = value {
            let offset = u16_bytes(bytes.len());
            bytes[field..field + 2].copy_from_slice(&offset);
            bytes.extend(axis(&value));
        }
    }
    if let Some(store) = store {
        let offset = u32::try_from(bytes.len()).unwrap().to_be_bytes();
        bytes[8..12].copy_from_slice(&offset);
        bytes.extend(store);
    }
    bytes
}

pub(crate) fn single_axis(tags: &[[u8; 4]], coordinates: &[i16]) -> Axis {
    Axis {
        tags: tags.to_vec(),
        scripts: vec![Script {
            tag: *b"latn",
            default_index: 0,
            coordinates: coordinates.iter().copied().map(Coord::Design).collect(),
        }],
    }
}

pub(crate) fn full_base() -> Vec<u8> {
    let tags = [*b"ideo", *b"idtp", *b"romn"];
    let mut horizontal = single_axis(&tags, &[-200, 800, 25]);
    horizontal.scripts[0].default_index = 2;
    let mut vertical = single_axis(&tags, &[0, 1200, 350]);
    vertical.scripts[0].default_index = 2;
    base(0, Some(horizontal), Some(vertical), None)
}

pub(crate) fn device(start: u16, end: u16, format: u16, words: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [start, end, format] {
        bytes.extend(value.to_be_bytes());
    }
    for word in words {
        bytes.extend(word.to_be_bytes());
    }
    bytes
}

pub(crate) fn variation_index(outer: u16, inner: u16) -> Vec<u8> {
    device(outer, inner, 0x8000, &[])
}

pub(crate) fn one_axis_store(delta: i16) -> Vec<u8> {
    region_store(&[[0, 16384, 16384]], delta)
}

pub(crate) fn region_store(axes: &[[i16; 3]], delta: i16) -> Vec<u8> {
    let mut bytes = Vec::from(1u16.to_be_bytes());
    bytes.extend(12u32.to_be_bytes());
    bytes.extend(1u16.to_be_bytes());
    bytes.extend(u32::try_from(16 + 6 * axes.len()).unwrap().to_be_bytes());
    bytes.extend(u16_bytes(axes.len()));
    bytes.extend(1u16.to_be_bytes());
    for axis in axes {
        for value in axis {
            bytes.extend(value.to_be_bytes());
        }
    }
    // One row, one signed16 word, one region index0.
    for value in [1u16, 1, 1, 0] {
        bytes.extend(value.to_be_bytes());
    }
    bytes.extend(delta.to_be_bytes());
    bytes
}

pub(crate) fn two_axis_ahem() -> Vec<u8> {
    // Existing real X tuple, adding an embedded neutral second peak.
    let mut tuple = vec![
        0, 1, 0, 10, 0, 11, 0xa0, 0, 0x40, 0, 0, 7, 0, 100, 100, 0, 0, 0, 0, 0, 0x87,
    ];
    tuple.insert(10, 0);
    tuple.insert(11, 0);
    tuple[2..4].copy_from_slice(&12u16.to_be_bytes());
    let font = test_fonts::variable_ahem_with_tuple(&tuple);
    let mut fvar = table(&font, *b"fvar").to_vec();
    fvar[8..10].copy_from_slice(&2u16.to_be_bytes());
    fvar[14..16].copy_from_slice(&12u16.to_be_bytes());
    fvar.extend(*b"wdth");
    for value in [0i32, 0, 65536] {
        fvar.extend(value.to_be_bytes());
    }
    fvar.extend(0u16.to_be_bytes());
    fvar.extend(256u16.to_be_bytes());
    let mut gvar = table(&font, *b"gvar").to_vec();
    gvar[4..6].copy_from_slice(&2u16.to_be_bytes());
    replace_tables(&font, vec![(*b"fvar", fvar), (*b"gvar", gvar)])
}

pub(crate) fn variation_base() -> Vec<u8> {
    let horizontal = Axis {
        tags: vec![*b"ideo", *b"idtp"],
        scripts: vec![Script {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![
                Coord::Design(0),
                Coord::Adjusted {
                    coordinate: 1000,
                    adjustment: Some(variation_index(0, 0)),
                },
            ],
        }],
    };
    base(1, Some(horizontal), None, Some(one_axis_store(3)))
}

pub(crate) fn metric_discriminator(use_typo: bool, hhea_gap: i16, typo_gap: i16) -> Vec<u8> {
    let mut hhea = table(AHEM, *b"hhea").to_vec();
    let mut os2 = table(AHEM, *b"OS/2").to_vec();
    for (offset, value) in [(4, 900i16), (6, -300), (8, hhea_gap)] {
        hhea[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    assert!(os2.len() >= 96);
    os2[..2].copy_from_slice(&4u16.to_be_bytes());
    let flags = (u16_at(&os2, 62) & !0x80) | if use_typo { 0x80 } else { 0 };
    os2[62..64].copy_from_slice(&flags.to_be_bytes());
    for (offset, value) in [(68, 700i16), (70, -200), (72, typo_gap)] {
        os2[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    replace_tables(AHEM, vec![(*b"hhea", hhea), (*b"OS/2", os2)])
}

pub(crate) fn variable_metric_ahem() -> Vec<u8> {
    let font = test_fonts::variable_ahem();
    let mut os2 = table(&font, *b"OS/2").to_vec();
    os2[..2].copy_from_slice(&4u16.to_be_bytes());
    let flags = u16_at(&os2, 62) | 0x80;
    os2[62..64].copy_from_slice(&flags.to_be_bytes());
    // MVAR1.0: one hasc record, outer0/inner0, actual store at20.
    let mut mvar = Vec::new();
    for value in [1u16, 0, 0, 8, 1, 20] {
        mvar.extend(value.to_be_bytes());
    }
    mvar.extend(*b"hasc");
    mvar.extend(0u16.to_be_bytes());
    mvar.extend(0u16.to_be_bytes());
    mvar.extend(one_axis_store(100));
    replace_tables(&font, vec![(*b"OS/2", os2), (*b"MVAR", mvar)])
}

pub(crate) fn compound_ahem() -> Vec<u8> {
    compound_fixture(false)
}

pub(crate) fn point_attached_ahem() -> Vec<u8> {
    compound_fixture(true)
}

fn compound_fixture(point_attached: bool) -> Vec<u8> {
    let old_loca = table(AHEM, *b"loca");
    let old_glyf = table(AHEM, *b"glyf");
    assert_eq!(u16_at(table(AHEM, *b"head"), 50), 0);
    let mut maxp = table(AHEM, *b"maxp").to_vec();
    let count = usize::from(u16_at(&maxp, 4));
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    for glyph in 0..count {
        loca.extend(u16_bytes(glyf.len() / 2));
        if glyph == 35 {
            // The first real X58 must use XY placement. A second component
            // may attach its actual point0 to the first X's actual point0.
            for value in [-1i16, 0, -200, 1000, 800] {
                glyf.extend(value.to_be_bytes());
            }
            let first_flags = if point_attached { 0x0023u16 } else { 0x0003 };
            for value in [first_flags, 58, 0, 0] {
                glyf.extend(value.to_be_bytes());
            }
            if point_attached {
                for value in [0x0001u16, 58, 0, 0] {
                    glyf.extend(value.to_be_bytes());
                }
            }
        } else {
            let start = usize::from(u16_at(old_loca, 2 * glyph)) * 2;
            let end = usize::from(u16_at(old_loca, 2 * (glyph + 1))) * 2;
            glyf.extend_from_slice(&old_glyf[start..end]);
        }
        assert!(glyf.len().is_multiple_of(2));
    }
    loca.extend(u16_bytes(glyf.len() / 2));
    let points = if point_attached { 8 } else { 4 };
    let components = if point_attached { 2 } else { 1 };
    for (offset, minimum) in [(10, points), (12, components), (28, components), (30, 1)] {
        let value = u16_at(&maxp, offset).max(minimum);
        maxp[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    replace_tables(
        AHEM,
        vec![(*b"glyf", glyf), (*b"loca", loca), (*b"maxp", maxp)],
    )
}
