//! Repository-authored OpenType tables over the unchanged licensed Ahem resource.
//! These fixtures exercise the real registered-font providers, not fabricated
//! glyph output. Every substituted glyph already exists in Ahem's maxp/glyf.

use crate::test_fonts::replace_tables;

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

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
