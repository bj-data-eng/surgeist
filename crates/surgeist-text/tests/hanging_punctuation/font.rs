//! Narrow original adaptation of ../inline_boundary_edges/font.rs over the
//! same retained CC0/public-domain Ahem. No existing helper or font is modified.
//! A/period and opening/acute share the real 35+35→58 ccmp lookup; source
//! Unicode properties remain their actual character properties. This is a
//! provider fixture, not a natural punctuation or combining typeface.
const AHEM: &[u8] = include_bytes!("../fixtures/fonts/ahem/Ahem.ttf");

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn short(value: usize) -> u16 {
    u16::try_from(value).unwrap()
}

pub fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let read16 = |at| u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap());
    let read32 = |at| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    let record = (0..usize::from(read16(4)))
        .map(|index| 12 + index * 16)
        .find(|record| bytes[*record..*record + 4] == tag)
        .unwrap();
    let start = usize::try_from(read32(record + 8)).unwrap();
    let len = usize::try_from(read32(record + 12)).unwrap();
    &bytes[start..start + len]
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn rebuild(replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let count = u16::from_be_bytes(AHEM[4..6].try_into().unwrap());
    let mut tables = (0..usize::from(count))
        .map(|index| {
            let record = 12 + index * 16;
            let tag = AHEM[record..record + 4].try_into().unwrap();
            (tag, table(AHEM, tag).to_vec())
        })
        .collect::<Vec<_>>();
    for (tag, data) in replacements {
        if let Some(entry) = tables.iter_mut().find(|entry| entry.0 == tag) {
            entry.1 = data;
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
    let mut head = None;
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        if tag == *b"head" {
            data[8..12].fill(0);
            head = Some(output.len());
        }
        let record = 12 + index * 16;
        let offset = u32::try_from(output.len()).unwrap();
        output[record..record + 4].copy_from_slice(&tag);
        output[record + 4..record + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        output[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
        output[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
    }
    let head = head.unwrap();
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&output));
    output[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&output), 0xb1b0_afba);
    output
}

fn cmap(entries: &[(u32, u32)]) -> Vec<u8> {
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

fn layout(script: [u8; 4], mut features: Vec<([u8; 4], u16, Vec<u8>)>) -> Vec<u8> {
    features.sort_by_key(|feature| feature.0);
    let count = short(features.len());
    let mut scripts = words(&[1]);
    scripts.extend(script);
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, count]));
    for index in 0..count {
        scripts.extend(words(&[index]));
    }
    let mut list = words(&[count]);
    for (index, feature) in features.iter().enumerate() {
        list.extend(feature.0);
        list.extend(words(&[short(2 + 6 * features.len() + index * 6)]));
    }
    for index in 0..count {
        list.extend(words(&[0, 1, index]));
    }
    let mut lookups = words(&[count]);
    lookups.resize(2 + 2 * features.len(), 0);
    for (index, (_, kind, subtable)) in features.iter().enumerate() {
        let start = short(lookups.len());
        lookups[2 + index * 2..4 + index * 2].copy_from_slice(&start.to_be_bytes());
        lookups.extend(words(&[*kind, 0, 1, 8]));
        lookups.extend(subtable);
    }
    let mut output = words(&[
        1,
        0,
        10,
        short(10 + scripts.len()),
        short(10 + scripts.len() + list.len()),
    ]);
    output.extend(scripts);
    output.extend(list);
    output.extend(lookups);
    output
}

/// One real full-em X58 draw. Distinct A/period TCUs receive the existing
/// provider owner's equal 5/5 contributions at size10; opening+acute is one
/// complete TCU with total10. Each exact source is qualified before use.
pub fn punctuation_ligature() -> Vec<u8> {
    rebuild(vec![
        (
            *b"cmap",
            cmap(&[
                (0x20, 3),
                (0x22, 35),
                (0x28, 35),
                (0x29, 11),
                (0x2e, 35),
                (0x41, 35),
                (0x42, 36),
                (0x301, 35),
                (0x20dd, 35),
                (0x3000, 3),
                (0x4e00, 36),
            ]),
        ),
        (
            *b"GSUB",
            layout(
                *b"DFLT",
                vec![(
                    *b"ccmp",
                    4,
                    words(&[1, 18, 1, 8, 1, 4, 58, 2, 35, 1, 1, 35]),
                )],
            ),
        ),
    ])
}
