//! Repository-authored OpenType tables over the unchanged licensed Ahem resource.
//! These fixtures exercise the real registered-font providers, not fabricated
//! glyph output. Every substituted glyph already exists in Ahem's maxp/glyf.

use crate::test_fonts::replace_tables;

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

#[path = "casing.rs"]
mod casing;
#[path = "kana.rs"]
mod kana;

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

/// Fixed dot/Greek coverage over licensed Ahem for literal None baselines.
/// No casing is performed here. Marks100/101/102 have zero raw advance and
/// existing nonempty outlines; mark-to-base anchors are (100,200)/(400,600).
pub(crate) fn casing_ahem() -> Vec<u8> {
    let mappings = [
        (0x20, 3),
        (0x41, 35),
        (0x49, 43),
        (0x58, 58),
        (0x69, 75),
        (0xcd, 168),
        (0xed, 115),
        (0x130, 36),
        (0x131, 178),
        (0x301, 102),
        (0x307, 100),
        (0x308, 101),
        (0x386, 154),
        (0x389, 165),
        (0x390, 110),
        (0x391, 35),
        (0x394, 210),
        (0x397, 36),
        (0x399, 43),
        (0x39a, 45),
        (0x39f, 49),
        (0x3a3, 53),
        (0x3aa, 55),
        (0x3ac, 105),
        (0x3ae, 106),
        (0x3af, 107),
        (0x3b1, 67),
        (0x3b4, 70),
        (0x3b7, 71),
        (0x3b9, 75),
        (0x3ba, 77),
        (0x3bf, 81),
        (0x3c2, 84),
        (0x3c3, 85),
        (0x3ca, 109),
    ];
    let bases = [36, 43, 53, 55, 75, 84, 85];
    let mut gdef = words(&[1, 0, 12, 0, 0, 0, 2, 8]);
    for base in bases {
        gdef.extend(words(&[base, base, 1]));
    }
    gdef.extend(words(&[100, 102, 3]));
    // MarkBasePos format1: three marks, one class, seven actual bases.
    let mut subtable = words(&[1, 102, 112, 1, 12, 44]);
    subtable.extend(words(&[3, 0, 14, 0, 20, 0, 26]));
    for _ in 0..3 {
        subtable.extend(words(&[1, 100, 200]));
    }
    subtable.extend(words(&[7, 16, 22, 28, 34, 40, 46, 52]));
    for _ in bases {
        subtable.extend(words(&[1, 400, 600]));
    }
    subtable.extend(words(&[1, 3, 100, 101, 102]));
    subtable.extend(words(&[1, 7]));
    subtable.extend(words(&bases));
    let mut hmtx = crate::test_fonts::table(AHEM, *b"hmtx").to_vec();
    for glyph in [100, 101, 102] {
        hmtx[glyph * 4..glyph * 4 + 2].copy_from_slice(&0_u16.to_be_bytes());
    }
    replace_tables(
        AHEM,
        vec![
            (*b"cmap", cmap(&mappings)),
            (*b"hmtx", hmtx),
            (*b"GDEF", gdef),
            (
                *b"GPOS",
                layout_table(*b"DFLT", vec![(*b"mark", 4, subtable)]),
            ),
        ],
    )
}

/// Fixed cmap for A/C and small hiragana KO/full-size KO over licensed Ahem.
/// Original U+1B132 uses p(82),1000 units; U+3053 uses B(36),500 units.
/// These distinguish real source/font baselines without transforming any text.
pub(crate) fn kana_ahem() -> Vec<u8> {
    let mut hmtx = crate::test_fonts::table(AHEM, *b"hmtx").to_vec();
    hmtx[36 * 4..36 * 4 + 2].copy_from_slice(&500_u16.to_be_bytes());
    replace_tables(
        AHEM,
        vec![
            (
                *b"cmap",
                cmap(&[(0x41, 35), (0x43, 37), (0x1b132, 82), (0x3053, 36)]),
            ),
            (*b"hmtx", hmtx),
        ],
    )
}

/// Real mark-to-base fixture: transformed KO uses base35, original small KO82.
/// Acute100 retains the same actual (400,600)/(100,200) anchors as mark_ahem.
pub(crate) fn kana_mark_ahem() -> Vec<u8> {
    replace_tables(
        &mark_ahem(),
        vec![(
            *b"cmap",
            cmap(&[
                (0x41, 35),
                (0x43, 37),
                (0x1b132, 82),
                (0x3053, 35),
                (0x301, 100),
            ]),
        )],
    )
}

/// GSUB ccmp: original Ahem A(35) A(35) -> X(58), one 1000-unit glyph.
pub(crate) fn ligature_ahem() -> Vec<u8> {
    let subtable = words(&[1, 18, 1, 8, 1, 4, 58, 2, 35, 1, 1, 35]);
    replace_tables(
        AHEM,
        vec![(
            *b"GSUB",
            layout_table(*b"DFLT", vec![(*b"ccmp", 4, subtable)]),
        )],
    )
}

/// A real optional provider ligature with caller-owned test cmap/sequence.
pub(crate) fn range_ligature_ahem(components: &[u16], mappings: &[(u32, u32)]) -> Vec<u8> {
    let mut data = vec![
        1,
        offset((7 + components.len()) * 2),
        1,
        8,
        1,
        4,
        58,
        offset(components.len()),
    ];
    data.extend_from_slice(&components[1..]);
    data.extend_from_slice(&[1, 1, components[0]]);
    replace_tables(
        AHEM,
        vec![
            (*b"cmap", cmap(mappings)),
            (
                *b"GSUB",
                layout_table(*b"DFLT", vec![(*b"ccmp", 4, words(&data))]),
            ),
        ],
    )
}

/// GPOS mark-to-base: base anchor(400,600), mark anchor(100,200).
/// At size10/UPEM1000, the actual mark is at logical(3,-4), advance0.
pub(crate) fn mark_ahem() -> Vec<u8> {
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

fn single_substitution(input: &[u16], output: &[u16]) -> Vec<u8> {
    assert_eq!(input.len(), output.len());
    let mut table = words(&[2, offset(6 + output.len() * 2), offset(output.len())]);
    table.extend(words(output));
    table.extend(words(&[1, offset(input.len())]));
    table.extend(words(input));
    table
}

/// Arabic joining uses real Unicode code points and the arab script's actual
/// isol/init/medi/fina masks. Beh's literal form glyphs:35/100/277/82.
pub(crate) fn arabic_ahem() -> Vec<u8> {
    let features = vec![
        (*b"fina", 1, single_substitution(&[35, 58], &[82, 274])),
        (*b"init", 1, single_substitution(&[35], &[100])),
        (*b"isol", 1, single_substitution(&[35], &[35])),
        (*b"medi", 1, single_substitution(&[35], &[277])),
    ];
    replace_tables(
        AHEM,
        vec![
            (
                *b"cmap",
                cmap(&[
                    (0x20, 3),
                    (0x627, 58),
                    (0x628, 35),
                    (0x200c, 253),
                    (0x200d, 253),
                ]),
            ),
            (*b"GSUB", layout_table(*b"arab", features)),
        ],
    )
}

/// Original Arabic GSUB with a real nonjoining HAMZA for lexical padding.
pub(crate) fn arabic_nonjoining_ahem() -> Vec<u8> {
    replace_tables(
        &arabic_ahem(),
        vec![(
            *b"cmap",
            cmap(&[
                (0x20, 3),
                (0x621, 36),
                (0x627, 58),
                (0x628, 35),
                (0x200c, 253),
                (0x200d, 253),
            ]),
        )],
    )
}

fn pair_adjustment(first: u16, second: u16) -> Vec<u8> {
    // PairPos format1, ValueFormat1=xAdvance, no second value record.
    // Coverage and PairSet each contain exactly one actual glyph.
    words(&[
        1,
        18,
        4,
        0,
        1,
        12,
        1,
        second,
        u16::from_be_bytes((-500_i16).to_be_bytes()),
        1,
        1,
        first,
    ])
}

/// GPOS kern: A35 before B36 has xAdvance -500 at UPEM1000.
/// At20px the joint AB advance is30; independently shaped A and B total40.
pub(crate) fn pair_adjusted_ahem() -> Vec<u8> {
    replace_tables(
        AHEM,
        vec![(
            *b"GPOS",
            layout_table(*b"DFLT", vec![(*b"kern", 2, pair_adjustment(35, 36))]),
        )],
    )
}

/// Arabic GSUB is unchanged. Nonjoining HAMZA U+0621 maps to B36; only
/// original medial Beh277 followed by36 receives the -500-unit kern.
/// This distinguishes preserving the original action before joint GPOS from
/// either accepting presentation-derived final82 or splitting the two roles.
pub(crate) fn arabic_pair_adjusted_ahem() -> Vec<u8> {
    replace_tables(
        &arabic_ahem(),
        vec![
            (
                *b"cmap",
                cmap(&[
                    (0x20, 3),
                    (0x621, 36),
                    (0x627, 58),
                    (0x628, 35),
                    (0x62c, 36),
                    (0x200c, 253),
                    (0x200d, 253),
                ]),
            ),
            (
                *b"GPOS",
                layout_table(*b"arab", vec![(*b"kern", 2, pair_adjustment(277, 36))]),
            ),
        ],
    )
}

/// SPACE(3) has500 units; Latin rtla substitutes SPACE→.notdef(0),1000 units.
/// rtla also substitutes ZERO(19)→SPACE(3), so erroneous RTL ZERO measurement is observable.
pub(crate) fn directional_space_ahem() -> Vec<u8> {
    let mut hmtx = crate::test_fonts::table(AHEM, *b"hmtx").to_vec();
    hmtx[12..14].copy_from_slice(&500_u16.to_be_bytes());
    replace_tables(
        AHEM,
        vec![
            (*b"hmtx", hmtx),
            (
                *b"GSUB",
                layout_table(
                    *b"latn",
                    vec![(*b"rtla", 1, words(&[2, 10, 2, 0, 3, 1, 2, 3, 19]))],
                ),
            ),
        ],
    )
}
