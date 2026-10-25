//! Real SFNT/table stimuli for the DI contract, derived from retained Ahem.
use crate::hb::completion_fixture::{with_tables, words};
use crate::Shaper;
use alloc::{vec, vec::Vec};

pub(super) const A: u16 = 35;
pub(super) const B: u16 = 36;
pub(super) const D: u16 = 100;
pub(super) const R: u16 = 58;
pub(super) const Q: u16 = 277;
pub(super) const SPACE: u16 = 274;
const AHEM: &[u8] =
    include_bytes!("../../../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn short(n: usize) -> u16 {
    u16::try_from(n).unwrap()
}
fn longs(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

fn table(tag: &[u8; 4]) -> &[u8] {
    let count = u16::from_be_bytes(AHEM[4..6].try_into().unwrap()) as usize;
    let r = AHEM[12..12 + count * 16]
        .chunks_exact(16)
        .find(|r| &r[..4] == tag)
        .unwrap();
    let start = u32::from_be_bytes(r[8..12].try_into().unwrap()) as usize;
    let len = u32::from_be_bytes(r[12..16].try_into().unwrap()) as usize;
    &AHEM[start..start + len]
}

pub(super) fn cmap(di: Option<char>, uvs: bool) -> Vec<u8> {
    let mut entries = vec![
        (32, SPACE),
        (65, A),
        (66, B),
        (67, D),
        (81, Q),
        (82, R),
        (0x25cc, Q),
    ];
    if let Some(di) = di {
        entries.push((u32::from(di), D));
    }
    entries.sort_unstable();
    let mut map = words(&[12, 0]);
    map.extend(longs(&[
        16 + entries.len() as u32 * 12,
        0,
        entries.len() as u32,
    ]));
    for (cp, gid) in entries {
        map.extend(longs(&[cp, cp, u32::from(gid)]));
    }
    let mut cmap = words(&[0, if uvs { 2 } else { 1 }, 3, 10]);
    cmap.extend(longs(&[if uvs { 20 } else { 12 }]));
    if uvs {
        cmap.extend(words(&[0, 5]));
        cmap.extend(longs(&[20 + map.len() as u32]));
    }
    cmap.extend(map);
    if uvs {
        // format14: explicit A+FE0F -> R, no default range and no nominal selector required.
        cmap.extend(words(&[14]));
        cmap.extend(longs(&[30, 1]));
        cmap.extend([0, 0xfe, 0x0f]);
        cmap.extend(longs(&[0, 21, 1]));
        cmap.extend([0, 0, 65]);
        cmap.extend(words(&[R]));
    }
    cmap
}

pub(super) fn multiple(input: u16, output: &[u16]) -> (u16, Vec<u8>) {
    let mut data = words(&[1, short(10 + output.len() * 2), 1, 8, short(output.len())]);
    data.extend(words(output));
    data.extend(words(&[1, 1, input]));
    (2, data)
}

pub(super) fn ligature(first: u16, second: u16) -> (u16, Vec<u8>) {
    (4, words(&[1, 18, 1, 8, 1, 4, R, 2, second, 1, 1, first]))
}

pub(super) fn gsub(lookups: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut scripts = words(&[1]);
    scripts.extend(*b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0]));
    let mut features = words(&[1]);
    features.extend(*b"ccmp");
    features.extend(words(&[8, 0, short(lookups.len())]));
    features.extend(words(&(0..lookups.len()).map(short).collect::<Vec<_>>()));
    let mut list = words(&[short(lookups.len())]);
    let mut offset = 2 + 2 * lookups.len();
    for (_, bytes) in lookups {
        list.extend(words(&[short(offset)]));
        offset += 8 + bytes.len();
    }
    for (kind, bytes) in lookups {
        list.extend(words(&[*kind, 0, 1, 8]));
        list.extend(bytes);
    }
    let mut data = words(&[
        1,
        0,
        10,
        short(10 + scripts.len()),
        short(10 + scripts.len() + features.len()),
    ]);
    data.extend(scripts);
    data.extend(features);
    data.extend(list);
    data
}

pub(super) fn with_font(
    di: Option<char>,
    lookups: &[(u16, Vec<u8>)],
    morx: Option<&[u8]>,
    uvs: bool,
    f: impl FnOnce(&Shaper<'_>),
) {
    with_extra_tables(di, lookups, morx, uvs, &[], f);
}

pub(super) fn with_extra_tables(
    di: Option<char>,
    lookups: &[(u16, Vec<u8>)],
    morx: Option<&[u8]>,
    uvs: bool,
    extra: &[([u8; 4], &[u8])],
    f: impl FnOnce(&Shaper<'_>),
) {
    let cmap = cmap(di, uvs);
    let mut hmtx = table(b"hmtx").to_vec();
    for (gid, advance) in [
        (A, 1000u16),
        (B, 2000),
        (D, 700),
        (R, 1500),
        (Q, 2500),
        (SPACE, 1000),
    ] {
        hmtx[gid as usize * 4..gid as usize * 4 + 2].copy_from_slice(&advance.to_be_bytes());
    }
    let gsub = gsub(lookups);
    let mut tables = vec![(*b"cmap", cmap.as_slice()), (*b"hmtx", hmtx.as_slice())];
    if !lookups.is_empty() {
        tables.push((*b"GSUB", gsub.as_slice()));
    }
    if let Some(morx) = morx {
        tables.push((*b"morx", morx));
    }
    tables.extend_from_slice(extra);
    with_tables(&tables, f);
}

pub(super) fn morx(subtables: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut payload = Vec::new();
    for (kind, bytes) in subtables {
        let len = (12 + bytes.len() + 3) & !3;
        payload.extend(longs(&[len as u32, 0x2000_0000 | kind, 1]));
        payload.extend(bytes);
        while payload.len() % 4 != 0 {
            payload.push(0);
        }
    }
    let mut data = words(&[2, 0]);
    data.extend(longs(&[
        1,
        1,
        16 + payload.len() as u32,
        0,
        subtables.len() as u32,
    ]));
    data.extend(payload);
    data
}

/// Full contextual machine, one real current-glyph substitution, no mark effects.
pub(super) fn contextual(input: u16, output: u16) -> (u32, Vec<u8>) {
    let mut data = longs(&[5, 20, 28, 48, 64]);
    data.extend(words(&[8, input, 1, 4]));
    data.extend(words(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 1]));
    data.extend(words(&[0, 0, 0xffff, 0xffff, 0, 0, 0xffff, 0]));
    data.extend(longs(&[4])); // offsets relative to substitution table
    data.extend(words(&[8, input, 1, output]));
    (1, data)
}

/// Full insertion machine; its unchanged DI anchor remains a real live slot.
pub(super) fn insertion(marked: bool) -> (u32, Vec<u8>) {
    insertion_at(marked, D)
}

pub(super) fn insertion_at(marked: bool, trigger: u16) -> (u32, Vec<u8>) {
    let mut data = longs(&[5, 20, 28, 48, 64]);
    data.extend(words(&[8, trigger, 1, 4]));
    data.extend(words(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 1]));
    data.extend(words(&[0, 0, 0xffff, 0xffff]));
    if marked {
        // mark initially points at current input0; insert before that actual anchor.
        data.extend(words(&[0, 0x0401, 0xffff, 0]));
    } else {
        data.extend(words(&[0, 0x0820, 0, 0xffff]));
    }
    data.extend(words(&[R]));
    (5, data)
}

/// Exact A,D component selection; unrelated/skipped glyphs use class1.
pub(super) fn morx_ligature(second: u16) -> (u32, Vec<u8>) {
    // class lookup format6, A->4 D->5; three states (start/EOL/after A).
    let classes = words(&[6, 4, 2, 8, 1, 0, A, 4, second, 5]);
    let states = words(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 1, 3]);
    let entries = words(&[0, 0, 0, 2, 0x8000, 0, 2, 0, 0, 0, 0xa000, 0]);
    let action_at = 28 + classes.len() + states.len() + entries.len();
    let component_at = action_at + 8;
    let ligature_at = component_at + (D as usize + 1) * 2;
    let mut data = longs(&[
        6,
        28,
        (28 + classes.len()) as u32,
        (28 + classes.len() + states.len()) as u32,
        action_at as u32,
        component_at as u32,
        ligature_at as u32,
    ]);
    data.extend(classes);
    data.extend(states);
    data.extend(entries);
    data.extend(longs(&[0, 0x8000_0000]));
    data.extend(vec![0; (D as usize + 1) * 2]);
    data.extend(words(&[R]));
    (2, data)
}
