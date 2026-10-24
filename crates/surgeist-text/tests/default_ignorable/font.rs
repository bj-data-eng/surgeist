//! Real, checksum-correct SFNT derivatives of the retained licensed Ahem font.
//! Only cmap, hmtx and optional GSUB are authored here; glyf/loca stay unchanged.
//! See ../fixtures/fonts/ahem/PROVENANCE.md and COPYING for the source license.

const AHEM: &[u8] = include_bytes!("../fixtures/fonts/ahem/Ahem.ttf");

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
    let mut tables: Vec<_> = (0..usize::from(u16_at(AHEM, 4)))
        .map(|index| {
            let at = 12 + index * 16;
            let tag: [u8; 4] = AHEM[at..at + 4].try_into().unwrap();
            (tag, table(AHEM, tag).to_vec())
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
    output[..4].copy_from_slice(&AHEM[..4]);
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
    entries.sort_unstable();
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
    // DFLT/default LangSys selects ccmp with every lookup in source order.
    let scripts = {
        let mut data = words(&[1]);
        data.extend(*b"DFLT");
        data.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0]));
        data
    };
    let features = {
        let mut data = words(&[1]);
        data.extend(*b"ccmp");
        data.extend(words(&[8, 0, short(lookups.len())]));
        for index in 0..lookups.len() {
            data.extend(words(&[short(index)]));
        }
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
