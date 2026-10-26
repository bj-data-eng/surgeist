//! Real, checksum-correct SFNT derivatives of the retained licensed Ahem font.
//! Only cmap, hmtx and optional GSUB are authored here; glyf/loca stay unchanged.
//! See ../fixtures/fonts/ahem/PROVENANCE.md and COPYING for the source license.

const AHEM: &[u8] = include_bytes!("../fixtures/fonts/ahem/Ahem.ttf");

#[path = "morx.rs"]
mod morx;
pub use morx::{MorxAction, morx_fixture};

pub const A: u16 = 35;
pub const B: u16 = 36;
pub const D: u16 = 100;
pub const R: u16 = 58;
pub const Q: u16 = 277;
pub const SPACE: u16 = 274;

#[derive(Clone, Copy, Debug)]
pub enum Action {
    None,
    Delete,
    ReplaceThenDelete,
    PartialDelete,
    DeleteBoth,
    Consume,
    Skip,
}

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn short(value: usize) -> u16 {
    u16::try_from(value).unwrap()
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| 12 + index * 16)
        .find(|record| bytes[*record..*record + 4] == tag)
        .unwrap();
    let start = usize::try_from(u32_at(bytes, record + 8)).unwrap();
    let length = usize::try_from(u32_at(bytes, record + 12)).unwrap();
    &bytes[start..start + length]
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

// Narrow safe adaptation of the repository's src/test_fonts.rs SFNT writer.
fn replace_tables(replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    rewrite_tables(AHEM, replacements)
}

fn rewrite_tables(bytes: &[u8], replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let mut tables: Vec<_> = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| {
            let at = 12 + index * 16;
            let tag: [u8; 4] = bytes[at..at + 4].try_into().unwrap();
            (tag, table(bytes, tag).to_vec())
        })
        .collect();
    for (tag, data) in replacements {
        if let Some(existing) = tables.iter_mut().find(|entry| entry.0 == tag) {
            existing.1 = data;
        } else {
            tables.push((tag, data));
        }
    }
    tables.sort_by_key(|entry| entry.0);
    let count = tables.len();
    let mut output = vec![0; 12 + count * 16];
    output[..4].copy_from_slice(&bytes[..4]);
    output[4..6].copy_from_slice(&short(count).to_be_bytes());
    let power = count.next_power_of_two() / if count.is_power_of_two() { 1 } else { 2 };
    output[6..8].copy_from_slice(&short(power * 16).to_be_bytes());
    output[8..10].copy_from_slice(&short(power.ilog2() as usize).to_be_bytes());
    output[10..12].copy_from_slice(&short((count - power) * 16).to_be_bytes());
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        let at = 12 + index * 16;
        let start = u32::try_from(output.len()).unwrap();
        output[at..at + 4].copy_from_slice(&tag);
        output[at + 4..at + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        output[at + 8..at + 12].copy_from_slice(&start.to_be_bytes());
        output[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    while !output.len().is_multiple_of(4) {
        output.push(0);
    }
    let head = (0..count)
        .map(|index| 12 + index * 16)
        .find(|at| output[*at..*at + 4] == *b"head")
        .unwrap();
    let head = usize::try_from(u32_at(&output, head + 8)).unwrap();
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&output));
    output[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&output), 0xb1b0_afba);
    output
}

fn cmap(mapped_di: Option<char>) -> Vec<u8> {
    // Ordinary C aliases D's glyph, so literal input qualifies every table action.
    let mut entries = vec![
        (0x20, u32::from(SPACE)),
        (0x41, u32::from(A)),
        (0x42, u32::from(B)),
        (0x43, u32::from(D)),
        (0x51, u32::from(Q)),
        (0x52, u32::from(R)),
    ];
    if let Some(value) = mapped_di {
        entries.push((u32::from(value), u32::from(D)));
    }
    cmap_entries(&entries)
}

fn cmap_entries(entries: &[(u32, u32)]) -> Vec<u8> {
    let mut entries = entries.to_vec();
    entries.sort_unstable();
    assert!(entries.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let mut output = words(&[0, 1, 3, 10]);
    output.extend(12_u32.to_be_bytes());
    output.extend(words(&[12, 0]));
    output.extend(
        u32::try_from(16 + entries.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    output.extend(0_u32.to_be_bytes());
    output.extend(u32::try_from(entries.len()).unwrap().to_be_bytes());
    for (scalar, glyph) in entries {
        output.extend(scalar.to_be_bytes());
        output.extend(scalar.to_be_bytes());
        output.extend(glyph.to_be_bytes());
    }
    output
}

fn multiple(input: u16, output: &[u16]) -> Vec<u8> {
    // MultipleSubst format1 -> one Sequence table, including genuine length zero.
    let mut data = words(&[1, short(10 + output.len() * 2), 1, 8, short(output.len())]);
    data.extend(words(output));
    data.extend(words(&[1, 1, input]));
    data
}

fn ligature(first: u16, second: u16) -> Vec<u8> {
    words(&[1, 18, 1, 8, 1, 4, R, 2, second, 1, 1, first])
}

fn gsub(lookups: Vec<(u16, Vec<u8>)>) -> Vec<u8> {
    let enabled = (0..lookups.len()).map(short).collect::<Vec<_>>();
    gsub_selected(*b"DFLT", lookups, &enabled)
}

fn gsub_selected(script: [u8; 4], lookups: Vec<(u16, Vec<u8>)>, enabled: &[u16]) -> Vec<u8> {
    // DFLT/default LangSys selects ccmp with every lookup in source order.
    let scripts = {
        let mut data = words(&[1]);
        data.extend(script);
        data.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0]));
        data
    };
    let features = {
        let mut data = words(&[1]);
        data.extend(*b"ccmp");
        data.extend(words(&[8, 0, short(enabled.len())]));
        data.extend(words(enabled));
        data
    };
    let mut lookup_list = words(&[short(lookups.len())]);
    lookup_list.resize(2 + 2 * lookups.len(), 0);
    for (index, (kind, subtable)) in lookups.into_iter().enumerate() {
        let start = short(lookup_list.len());
        lookup_list[2 + index * 2..4 + index * 2].copy_from_slice(&start.to_be_bytes());
        lookup_list.extend(words(&[kind, 0, 1, 8]));
        lookup_list.extend(subtable);
    }
    let mut output = words(&[
        1,
        0,
        10,
        short(10 + scripts.len()),
        short(10 + scripts.len() + features.len()),
    ]);
    output.extend(scripts);
    output.extend(features);
    output.extend(lookup_list);
    output
}

pub fn fixture(mapped_di: Option<char>, action: Action) -> Vec<u8> {
    let mut hmtx = table(AHEM, *b"hmtx").to_vec();
    // UPEM1000, rendered size10: A10 B20 D7 R15 Q25 SPACE10.
    for (glyph, advance) in [
        (A, 1000_u16),
        (B, 2000),
        (D, 700),
        (R, 1500),
        (Q, 2500),
        (SPACE, 1000),
    ] {
        let at = usize::from(glyph) * 4;
        hmtx[at..at + 2].copy_from_slice(&advance.to_be_bytes());
    }
    assert_eq!(u16_at(table(AHEM, *b"head"), 18), 1000);
    let lookups = match action {
        Action::None => vec![],
        Action::Delete => vec![(2, multiple(D, &[]))],
        Action::ReplaceThenDelete => vec![(2, multiple(D, &[R])), (2, multiple(R, &[]))],
        Action::PartialDelete => vec![(2, multiple(D, &[R, Q])), (2, multiple(Q, &[]))],
        Action::DeleteBoth => vec![
            (2, multiple(D, &[R, Q])),
            (2, multiple(Q, &[])),
            (2, multiple(R, &[])),
        ],
        Action::Consume => vec![(4, ligature(A, D))],
        Action::Skip => vec![(4, ligature(A, B))],
    };
    let mut tables = vec![(*b"cmap", cmap(mapped_di)), (*b"hmtx", hmtx)];
    if !lookups.is_empty() {
        tables.push((*b"GSUB", gsub(lookups)));
    }
    replace_tables(tables)
}

/// A real native-shaper control proves which components the table consumed.
/// Preserve leaves unresolved DI's actual nominal glyph observable for this
/// control; product tests below still exercise Text's own required profile.
pub fn native_glyphs(bytes: &[u8], source: &str) -> Vec<(u32, i32)> {
    let font = harfrust::FontRef::new(bytes).unwrap();
    let data = harfrust::ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    let mut input = harfrust::UnicodeBuffer::new();
    input.push_str(source);
    input.set_direction(harfrust::Direction::LeftToRight);
    input.set_flags(harfrust::BufferFlags::PRESERVE_DEFAULT_IGNORABLES);
    input.guess_segment_properties();
    let shaped = shaper.shape(input, &[]);
    assert_eq!(shaped.shaping_status(), harfrust::ShapingStatus::Complete);
    shaped
        .glyph_infos()
        .iter()
        .zip(shaped.glyph_positions())
        .map(|(info, position)| (info.glyph_id, position.x_advance))
        .collect()
}

fn mappings() -> Vec<(u32, u32)> {
    vec![
        (0x20, u32::from(SPACE)),
        (0x41, u32::from(A)),
        (0x42, u32::from(B)),
        (0x43, u32::from(D)),
        (0x51, u32::from(Q)),
        (0x52, u32::from(R)),
    ]
}

/// cmap format14 explicitly supports only A + U+E0100 -> R58. The selector
/// deliberately has no nominal mapping, so a successful real UVS is the witness.
pub fn variation(supported: bool, mapped_selector: bool) -> Vec<u8> {
    let bytes = fixture(None, Action::None);
    let mut entries = mappings();
    if mapped_selector {
        entries.push((0xe0100, u32::from(D)));
    }
    let base = cmap_entries(&entries);
    if !supported {
        return rewrite_tables(&bytes, vec![(*b"cmap", base)]);
    }
    let mut format14 = words(&[14]);
    format14.extend(30_u32.to_be_bytes());
    format14.extend(1_u32.to_be_bytes());
    format14.extend([0x0e, 0x01, 0x00]); // selector U+E0100, UInt24
    format14.extend(0_u32.to_be_bytes()); // no default UVS table
    format14.extend(21_u32.to_be_bytes()); // non-default mapping table
    format14.extend(1_u32.to_be_bytes());
    format14.extend([0, 0, 0x41]);
    format14.extend(R.to_be_bytes());
    assert_eq!(format14.len(), 30);
    let mut cmap = words(&[0, 2, 0, 5]); // sorted platform0 variation encoding
    cmap.extend(u32::try_from(base.len() + 8).unwrap().to_be_bytes());
    cmap.extend(words(&[3, 10]));
    cmap.extend(20_u32.to_be_bytes());
    cmap.extend(&base[12..]);
    cmap.extend(format14);
    rewrite_tables(&bytes, vec![(*b"cmap", cmap)])
}

/// Genuine Mongolian ccmp: U1820(A35)+U180B(D100)->R58. The ordinary
/// U1821 also maps to D100 as a positive same-script nominal control. U180F
/// maps to a distinct visible Q277, so that unsupported selector cannot
/// accidentally match the supported selector's component glyph.
pub fn mongolian(supported: bool) -> Vec<u8> {
    let bytes = fixture(None, Action::None);
    let mut entries = mappings();
    entries.extend([
        (0x1820, u32::from(A)),
        (0x1821, u32::from(D)),
        (0x180b, u32::from(D)),
        (0x180f, u32::from(Q)),
    ]);
    let mut tables = vec![(*b"cmap", cmap_entries(&entries))];
    if supported {
        tables.push((
            *b"GSUB",
            gsub_selected(*b"mong", vec![(4, ligature(A, D))], &[0]),
        ));
    }
    rewrite_tables(&bytes, tables)
}

/// The four recognized fillers share a real nonzero blank glyph3/1800-unit
/// spacing resource. Ordinary F qualifies its actual blank outline/metrics.
pub fn fillers(mapped: bool) -> Vec<u8> {
    let bytes = fixture(None, Action::None);
    let mut entries = mappings();
    entries.push((0x46, 3));
    if mapped {
        entries.extend([0x115f, 0x1160, 0x3164, 0xffa0].map(|scalar| (scalar, 3)));
    }
    let mut hmtx = table(&bytes, *b"hmtx").to_vec();
    hmtx[12..14].copy_from_slice(&1800_u16.to_be_bytes());
    rewrite_tables(
        &bytes,
        vec![(*b"cmap", cmap_entries(&entries)), (*b"hmtx", hmtx)],
    )
}

fn single(input: u16, output: u16) -> Vec<u8> {
    words(&[2, 8, 1, output, 1, 1, input])
}

fn context_lookahead(input: u16, lookahead: u16, nested: u16) -> Vec<u8> {
    // ChainedContextSubst format3, one input and one lookahead coverage.
    words(&[
        3, 0, 1, 18, 1, 24, 1, 0, nested, 1, 1, input, 1, 1, lookahead,
    ])
}

fn context_backtrack(backtrack: u16, input: u16, nested: u16) -> Vec<u8> {
    // ChainedContextSubst format3, one backtrack and one input coverage.
    words(&[
        3, 1, 18, 1, 24, 0, 1, 0, nested, 1, 1, backtrack, 1, 1, input,
    ])
}

/// DI acts only as lookahead to change A->R. Nested lookup1 is deliberately
/// absent from the feature's direct lookup list, qualifying context-only input.
pub fn context_only() -> Vec<u8> {
    let bytes = fixture(Some('\u{2065}'), Action::None);
    rewrite_tables(
        &bytes,
        vec![(
            *b"GSUB",
            gsub_selected(
                *b"DFLT",
                vec![(6, context_lookahead(A, D, 1)), (1, single(A, R))],
                &[0],
            ),
        )],
    )
}

/// Equal U2065 scalars take different actual paths: after A they become R;
/// after B they disappear. Supplementary U1BCA0/Q remains unresolved before A.
pub fn repeated() -> Vec<u8> {
    let bytes = fixture(Some('\u{2065}'), Action::None);
    let mut entries = mappings();
    entries.extend([(0x2065, u32::from(D)), (0x1bca0, u32::from(Q))]);
    rewrite_tables(
        &bytes,
        vec![
            (*b"cmap", cmap_entries(&entries)),
            (
                *b"GSUB",
                gsub_selected(
                    *b"DFLT",
                    vec![
                        (6, context_backtrack(A, D, 1)),
                        (1, single(D, R)),
                        (6, context_backtrack(B, D, 3)),
                        (2, multiple(D, &[])),
                    ],
                    &[0, 2],
                ),
            ),
        ],
    )
}
