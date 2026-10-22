const AHEM: &[u8] = include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf");
const X: u32 = 58;

// Repository-authored variable Ahem fixture construction copied from Render's
// tests/prepared_outline.rs. The original licensed Ahem bytes are unchanged;
// see Render tests/fixtures/fonts/ahem/PROVENANCE.md for the actual one-axis
// contour-delta facts. This is real table data consumed by registered shaping,
// not a synthetic coordinate attestation or broad variable-font conformance.
pub(crate) fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
pub(crate) fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
pub(crate) fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + i * 16)
        .find(|r| bytes[*r..*r + 4] == tag)
        .unwrap();
    let offset = u32_at(bytes, record + 8) as usize;
    let len = u32_at(bytes, record + 12) as usize;
    &bytes[offset..offset + len]
}
pub(crate) fn replace_tables(bytes: &[u8], replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    rewrite_tables(bytes, replacements, &[])
}

pub(crate) fn remove_tables(bytes: &[u8], removed: &[[u8; 4]]) -> Vec<u8> {
    rewrite_tables(bytes, Vec::new(), removed)
}

fn rewrite_tables(
    bytes: &[u8],
    replacements: Vec<([u8; 4], Vec<u8>)>,
    removed: &[[u8; 4]],
) -> Vec<u8> {
    let mut tables: Vec<_> = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| {
            let r = 12 + i * 16;
            let tag: [u8; 4] = bytes[r..r + 4].try_into().unwrap();
            (tag, table(bytes, tag).to_vec())
        })
        .filter(|(tag, _)| !removed.contains(tag))
        .collect();
    for (tag, data) in replacements {
        if let Some(t) = tables.iter_mut().find(|t| t.0 == tag) {
            t.1 = data;
        } else {
            tables.push((tag, data));
        }
    }
    tables.sort_by_key(|t| t.0);
    let mut output = vec![0; 12 + tables.len() * 16];
    output[..4].copy_from_slice(&bytes[..4]);
    output[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    let power =
        tables.len().next_power_of_two() / if tables.len().is_power_of_two() { 1 } else { 2 };
    output[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    output[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    output[10..12].copy_from_slice(
        &u16::try_from((tables.len() - power) * 16)
            .unwrap()
            .to_be_bytes(),
    );
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        let r = 12 + i * 16;
        let offset = u32::try_from(output.len()).unwrap();
        output[r..r + 4].copy_from_slice(&tag);
        output[r + 8..r + 12].copy_from_slice(&offset.to_be_bytes());
        output[r + 12..r + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    output
}

// Authored derivative of licensed Ahem. One wght axis [0,1], default0. At +1,
// X's two right-edge points move +100 font units, while all phantom points and
// y deltas stay zero. Thus width is 1000 + coordinate*100 at UPEM1000.
pub(crate) fn variable_ahem() -> Vec<u8> {
    variable_ahem_with_tuple(&[
        0, 1, 0, 10, 0, 11, 0xA0, 0, 0x40, 0, 0, 7, 0, 100, 100, 0, 0, 0, 0, 0, 0x87,
    ])
}
pub(crate) fn variable_ahem_with_tuple(tuple: &[u8]) -> Vec<u8> {
    let mut fvar = vec![];
    fvar.extend(0x00010000u32.to_be_bytes());
    for value in [16u16, 2, 1, 20, 0, 8] {
        fvar.extend(value.to_be_bytes());
    }
    fvar.extend(*b"wght");
    for value in [0i32, 0, 65536] {
        fvar.extend(value.to_be_bytes());
    }
    fvar.extend(0u16.to_be_bytes());
    fvar.extend(256u16.to_be_bytes());
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let data_offset = 20 + (count + 1) * 4;
    let mut gvar = vec![];
    gvar.extend(0x00010000u32.to_be_bytes());
    gvar.extend(1u16.to_be_bytes());
    gvar.extend(0u16.to_be_bytes());
    gvar.extend(u32::try_from(data_offset).unwrap().to_be_bytes());
    gvar.extend(u16::try_from(count).unwrap().to_be_bytes());
    gvar.extend(1u16.to_be_bytes());
    gvar.extend(u32::try_from(data_offset).unwrap().to_be_bytes());
    for i in 0..=count {
        gvar.extend(
            (if i <= X as usize {
                0u32
            } else {
                tuple.len() as u32
            })
            .to_be_bytes(),
        );
    }
    gvar.extend_from_slice(tuple);
    replace_tables(AHEM, vec![(*b"fvar", fvar), (*b"gvar", gvar)])
}

pub(crate) fn two_face_collection(second: Vec<u8>) -> Vec<u8> {
    let mut collection = Vec::from(&b"ttcf\0\x01\0\0\0\0\0\x02"[..]);
    let first_offset = 20_u32;
    let second_offset = first_offset
        .checked_add(u32::try_from(AHEM.len()).expect("fixture fits a font offset"))
        .expect("two fixtures fit font offsets");
    collection.extend_from_slice(&first_offset.to_be_bytes());
    collection.extend_from_slice(&second_offset.to_be_bytes());
    for (bytes, offset) in [(AHEM, first_offset), (second.as_slice(), second_offset)] {
        let start = collection.len();
        collection.extend_from_slice(bytes);
        let tables = usize::from(u16::from_be_bytes([bytes[4], bytes[5]]));
        for table in 0..tables {
            let field = start + 12 + table * 16 + 8;
            let original = u32::from_be_bytes(
                collection[field..field + 4]
                    .try_into()
                    .expect("four-byte table offset"),
            );
            collection[field..field + 4].copy_from_slice(
                &original
                    .checked_add(offset)
                    .expect("fixture table offset fits")
                    .to_be_bytes(),
            );
        }
    }
    collection
}
