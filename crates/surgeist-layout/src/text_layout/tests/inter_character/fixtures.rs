//! Licensed Ahem derivatives with independently inspected real liga tables.
pub(super) const AHEM: &[u8] =
    include_bytes!("../../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: &[u8; 4]) -> (usize, usize, usize) {
    let record = (0..usize::from(word(bytes, 4)))
        .map(|index| 12 + 16 * index)
        .find(|&record| &bytes[record..record + 4] == tag)
        .unwrap();
    (
        record,
        dword(bytes, record + 8) as usize,
        dword(bytes, record + 12) as usize,
    )
}
fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |sum, chunk| {
        let mut value = [0; 4];
        value[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(value))
    })
}

fn cmap(bytes: &[u8], scalar: u16) -> u16 {
    let (_, base, _) = table(bytes, b"cmap");
    let sub = (0..usize::from(word(bytes, base + 2)))
        .map(|index| base + dword(bytes, base + 8 + index * 8) as usize)
        .find(|&sub| word(bytes, sub) == 4)
        .expect("licensed Ahem has a BMP format4 cmap");
    let segments = usize::from(word(bytes, sub + 6) / 2);
    for index in 0..segments {
        let end = word(bytes, sub + 14 + 2 * index);
        let start = word(bytes, sub + 16 + 2 * segments + 2 * index);
        if start <= scalar && scalar <= end {
            let delta = word(bytes, sub + 16 + 4 * segments + 2 * index);
            let offset_at = sub + 16 + 6 * segments + 2 * index;
            let offset = usize::from(word(bytes, offset_at));
            if offset == 0 {
                return scalar.wrapping_add(delta);
            }
            let glyph = word(bytes, offset_at + offset + 2 * usize::from(scalar - start));
            return if glyph == 0 {
                0
            } else {
                glyph.wrapping_add(delta)
            };
        }
    }
    0
}

pub(super) fn qualify_metrics(bytes: &[u8], x_advance: u16) {
    let (_, head, _) = table(bytes, b"head");
    let (_, hhea, _) = table(bytes, b"hhea");
    let (_, hmtx, _) = table(bytes, b"hmtx");
    assert_eq!(word(bytes, head + 18), 1000);
    assert!(word(bytes, hhea + 34) > 58);
    assert_eq!(word(bytes, hhea + 10), x_advance.max(1000));
    for (scalar, glyph, advance) in [
        (0x41, 35, 1000),
        (0x42, 36, 1000),
        (0x43, 37, 1000),
        (0x58, 58, x_advance),
    ] {
        assert_eq!(cmap(bytes, scalar), glyph);
        assert_eq!(word(bytes, hmtx + usize::from(glyph) * 4), advance);
    }
}

/// Real AA -> X lookup under liga, with optional-default LangSys admission.
pub(super) fn liga(x_advance: u16) -> Vec<u8> {
    assert!([1000, 3000].contains(&x_advance));
    let mut bytes = super::super::canonical::fonts::ligature_ahem();
    let (_, gsub, _) = table(&bytes, b"GSUB");
    let features = gsub + usize::from(word(&bytes, gsub + 6));
    assert_eq!(word(&bytes, features), 1);
    assert_eq!(&bytes[features + 2..features + 6], b"ccmp");
    bytes[features + 2..features + 6].copy_from_slice(b"liga");
    let (_, hmtx, _) = table(&bytes, b"hmtx");
    bytes[hmtx + 58 * 4..hmtx + 58 * 4 + 2].copy_from_slice(&x_advance.to_be_bytes());
    let (_, hhea, _) = table(&bytes, b"hhea");
    bytes[hhea + 10..hhea + 12].copy_from_slice(&x_advance.max(1000).to_be_bytes());
    let (_, head, _) = table(&bytes, b"head");
    bytes[head + 8..head + 12].fill(0);
    for tag in [b"GSUB", b"hmtx", b"hhea"] {
        let (record, offset, length) = table(&bytes, tag);
        let sum = checksum(&bytes[offset..offset + length]);
        bytes[record + 4..record + 8].copy_from_slice(&sum.to_be_bytes());
    }
    let adjustment = 0xb1b0_afbau32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    qualify_metrics(&bytes, x_advance);

    let scripts = gsub + usize::from(word(&bytes, gsub + 4));
    assert_eq!(word(&bytes, scripts), 1);
    assert_eq!(&bytes[scripts + 2..scripts + 6], b"DFLT");
    let script = scripts + usize::from(word(&bytes, scripts + 6));
    let language = script + usize::from(word(&bytes, script));
    assert_eq!(
        word(&bytes, language + 2),
        0xffff,
        "liga is not LangSys required"
    );
    assert_eq!(
        (word(&bytes, language + 4), word(&bytes, language + 6)),
        (1, 0)
    );
    assert_eq!(&bytes[features + 2..features + 6], b"liga");
    let feature = features + usize::from(word(&bytes, features + 6));
    assert_eq!(
        (word(&bytes, feature + 2), word(&bytes, feature + 4)),
        (1, 0)
    );
    let list = gsub + usize::from(word(&bytes, gsub + 8));
    assert_eq!(word(&bytes, list), 1);
    let lookup = list + usize::from(word(&bytes, list + 2));
    assert_eq!((word(&bytes, lookup), word(&bytes, lookup + 4)), (4, 1));
    let sub = lookup + usize::from(word(&bytes, lookup + 6));
    assert_eq!((word(&bytes, sub), word(&bytes, sub + 4)), (1, 1));
    let coverage = sub + usize::from(word(&bytes, sub + 2));
    assert_eq!(
        (
            word(&bytes, coverage),
            word(&bytes, coverage + 2),
            word(&bytes, coverage + 4)
        ),
        (1, 1, 35)
    );
    let set = sub + usize::from(word(&bytes, sub + 6));
    assert_eq!(word(&bytes, set), 1);
    let ligature = set + usize::from(word(&bytes, set + 2));
    assert_eq!(
        (
            word(&bytes, ligature),
            word(&bytes, ligature + 2),
            word(&bytes, ligature + 4)
        ),
        (58, 2, 35)
    );
    bytes
}
