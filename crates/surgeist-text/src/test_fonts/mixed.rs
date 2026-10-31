//! Mixed test inputs composed from the existing licensed Ahem table builders.
use crate::{baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical};

pub(crate) const SCRIPTS: &[[u8; 4]] = &[*b"arab", *b"hani", *b"hebr", *b"latn"];

pub(crate) fn axis(under: i16, over: i16) -> base::Axis {
    base::Axis {
        tags: vec![*b"ideo", *b"idtp"],
        scripts: SCRIPTS
            .iter()
            .map(|&tag| base::Script {
                tag,
                default_index: 0,
                coordinates: vec![base::Coord::Design(under), base::Coord::Design(over)],
            })
            .collect(),
    }
}

pub(crate) fn with_cmap(font: &[u8], mappings: &[(u32, u32)]) -> Vec<u8> {
    // Reuse the existing cmap encoder without retaining its unrelated GSUB.
    let carrier = crate::shaping_test_fonts::range_ligature_ahem(&[82, 82], mappings);
    sfnt::replace_tables(
        font,
        vec![(*b"cmap", sfnt::table(&carrier, *b"cmap").to_vec())],
    )
}

pub(crate) fn with_metrics(font: &[u8], horizontal: bool) -> Vec<u8> {
    let count = sfnt::u16_at(sfnt::table(font, *b"maxp"), 4);
    let native = vertical::with_native(font, 1200, 100, count);
    sfnt::replace_tables(
        &native,
        vec![(
            *b"BASE",
            base::base(
                0,
                horizontal.then(|| axis(100, 700)),
                Some(axis(0, 1200)),
                None,
            ),
        )],
    )
}

pub(crate) fn mappings() -> Vec<(u32, u32)> {
    vec![
        (0x20, 3),
        (0x30, 35),
        (0x31, 35),
        (0x32, 35),
        (0x41, 35),
        (0x42, 36),
        (0x43, 37),
        (0x58, 58),
        (0x301, 100),
        (0x5d0, 58),
        (0x5d1, 58),
        (0x1681, 58),
        (0x20dd, 100),
        (0x2329, 58),
        (0x232a, 58),
        (0x3001, 58),
        (0x3008, 58),
        (0x3009, 58),
        (0x3099, 100),
        (0x4e00, 58),
        (0xfffd, 58),
    ]
}

pub(crate) fn fixture(horizontal: bool) -> Vec<u8> {
    let font = crate::shaping_test_fonts::sideways_ahem();
    with_metrics(&with_cmap(&font, &mappings()), horizontal)
}
