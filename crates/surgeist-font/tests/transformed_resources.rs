use surgeist_font::{DecodeError, DecodeLimits, FontResource};

fn limits() -> DecodeLimits {
    DecodeLimits::new(2 << 20, 4 << 20, 2 << 20, 1000, 20, 32 << 20, 40 << 20)
}

fn base128(mut value: usize, out: &mut Vec<u8>) {
    let mut digits = [0u8; 5];
    let mut index = 4;
    digits[index] = (value & 127) as u8;
    while value > 127 {
        value >>= 7;
        index -= 1;
        digits[index] = (value as u8 & 127) | 128;
    }
    out.extend_from_slice(&digits[index..]);
}

// Independently authored RFC 7932 uncompressed metablocks, not a provider encoder.
fn literal_brotli(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return vec![6];
    }
    let mut out = Vec::new();
    for (index, block) in data.chunks(65536).enumerate() {
        let shift = if index == 0 { 4 } else { 3 };
        let header = ((block.len() - 1) << shift) | (1 << (shift + 16));
        out.extend_from_slice(&(header as u32).to_le_bytes()[..3]);
        out.extend_from_slice(block);
    }
    out.push(3);
    out
}

fn wrap(directory: &[u8], tables: usize, stream: &[u8], flavor: [u8; 4]) -> Vec<u8> {
    let compressed = literal_brotli(stream);
    let mut out = vec![0; 48];
    out[..4].copy_from_slice(b"wOF2");
    out[4..8].copy_from_slice(&flavor);
    out[12..14].copy_from_slice(&(tables as u16).to_be_bytes());
    out[20..24].copy_from_slice(&(compressed.len() as u32).to_be_bytes());
    out.extend_from_slice(directory);
    out.extend_from_slice(&compressed);
    let length = out.len() as u32;
    out[8..12].copy_from_slice(&length.to_be_bytes());
    out
}

// Independent WOFF2 transformed glyph data. Each glyph has a single contour
// whose on-curve points coincide at the origin. In the source SFNT, repeat
// flag 0x39 and a count encode each run of at most 256 unchanged coordinates.
fn repeated_points(glyphs: usize, points: usize, long: bool) -> Vec<u8> {
    let source_span = (14 + 2 * points.div_ceil(256) + 3) & !3;
    let original_length = source_span * glyphs;
    assert!(original_length <= 131070);
    let mut outline = vec![0, 0, 0, 0];
    outline.extend_from_slice(&(glyphs as u16).to_be_bytes());
    outline.extend_from_slice(&u16::from(long).to_be_bytes());
    let bbox = 4 * glyphs.div_ceil(32);
    for length in [
        glyphs * 2,
        glyphs * 3,
        glyphs * points,
        glyphs * (points + 1),
        0,
        bbox,
        0,
    ] {
        outline.extend_from_slice(&(length as u32).to_be_bytes());
    }
    for _ in 0..glyphs {
        outline.extend_from_slice(&[0, 1]);
    }
    for _ in 0..glyphs {
        outline.push(253);
        outline.extend_from_slice(&(points as u16).to_be_bytes());
    }
    outline.resize(outline.len() + glyphs * points, 0); // Zero-coordinate triplet flags.
    for _ in 0..glyphs {
        outline.resize(outline.len() + points, 0);
        outline.push(0); // Instruction count.
    }
    outline.resize(outline.len() + bbox, 0);
    let mut head = vec![0; 54];
    head[18..20].copy_from_slice(&1000u16.to_be_bytes());
    head[50..52].copy_from_slice(&u16::from(long).to_be_bytes());
    let mut maxp = vec![0; 32];
    maxp[..4].copy_from_slice(&0x10000u32.to_be_bytes());
    maxp[4..6].copy_from_slice(&(glyphs as u16).to_be_bytes());
    maxp[6..8].copy_from_slice(&(points as u16).to_be_bytes());
    maxp[9] = 1;
    let mut directory = vec![10];
    base128(original_length, &mut directory);
    base128(outline.len(), &mut directory);
    directory.push(11);
    base128((glyphs + 1) * if long { 4 } else { 2 }, &mut directory);
    base128(0, &mut directory);
    directory.push(1);
    base128(head.len(), &mut directory);
    directory.push(4);
    base128(maxp.len(), &mut directory);
    outline.extend(head);
    outline.extend(maxp);
    wrap(&directory, 4, &outline, [0, 1, 0, 0])
}

#[test]
fn repeated_point_flags_fit_the_original_short_loca_domain() {
    let resource = FontResource::decode(repeated_points(3, 44000, false), limits()).unwrap();
    let face = resource.face(0).unwrap();
    let glyf = face.table(*b"glyf").unwrap();
    let loca = face.table(*b"loca").unwrap();
    assert!(glyf.len() <= 131070);
    assert_eq!(loca.len(), 8);
    assert_eq!(&face.table(*b"head").unwrap()[50..52], &[0, 0]);
    for glyph in 0..3 {
        let at = |index: usize| {
            usize::from(u16::from_be_bytes(
                loca[index * 2..index * 2 + 2].try_into().unwrap(),
            )) * 2
        };
        let data = &glyf[at(glyph)..at(glyph + 1)];
        assert_eq!(&data[10..12], &43999u16.to_be_bytes());
        let (mut offset, mut points) = (14, 0);
        while points < 44000 {
            let flag = data[offset];
            offset += 1;
            assert_eq!(flag & !8, 0x31); // On-curve, unchanged x and y.
            let repeat = if flag & 8 != 0 {
                let value = data[offset];
                offset += 1;
                usize::from(value)
            } else {
                0
            };
            points += repeat + 1;
        }
        assert_eq!(points, 44000);
        assert!(data[offset..].iter().all(|byte| *byte == 0));
    }
}

#[test]
fn the_same_large_outline_has_a_working_long_loca_control() {
    let resource = FontResource::decode(repeated_points(3, 44000, true), limits()).unwrap();
    let face = resource.face(0).unwrap();
    assert_eq!(face.table(*b"loca").unwrap().len(), 16);
    assert_eq!(&face.table(*b"head").unwrap()[50..52], &[0, 1]);
}

#[test]
fn two_byte_glyph_alignment_keeps_many_small_glyphs_in_short_loca() {
    let glyphs = 7000usize;
    let bbox = 4 * glyphs.div_ceil(32);
    let mut outline = vec![0, 0, 0, 0];
    outline.extend_from_slice(&(glyphs as u16).to_be_bytes());
    outline.extend_from_slice(&0u16.to_be_bytes());
    for length in [glyphs * 2, glyphs, glyphs, glyphs * 2, 0, bbox, 0] {
        outline.extend_from_slice(&(length as u32).to_be_bytes());
    }
    for _ in 0..glyphs {
        outline.extend_from_slice(&[0, 1]);
    }
    outline.resize(outline.len() + glyphs, 1);
    outline.resize(outline.len() + glyphs, 23); // (+1,+1), on-curve triplet.
    outline.resize(outline.len() + glyphs * 2 + bbox, 0);
    let mut directory = vec![10];
    base128(glyphs * 18, &mut directory);
    base128(outline.len(), &mut directory);
    directory.push(11);
    base128((glyphs + 1) * 2, &mut directory);
    base128(0, &mut directory);
    let encoded = wrap(&directory, 2, &outline, [0, 1, 0, 0]);
    let resource = FontResource::decode(encoded, limits()).unwrap();
    let face = resource.face(0).unwrap();
    assert_eq!(face.table(*b"glyf").unwrap().len(), glyphs * 18);
    assert_eq!(
        &face.table(*b"glyf").unwrap()[..18],
        &[0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0x37, 1, 1, 0]
    );
    let loca = face.table(*b"loca").unwrap();
    assert_eq!(
        usize::from(u16::from_be_bytes(
            loca[glyphs * 2..glyphs * 2 + 2].try_into().unwrap()
        )) * 2,
        glyphs * 18
    );
}

#[test]
fn unknown_single_font_flavor_has_an_unsupported_error() {
    let mut encoded = repeated_points(2, 1, false);
    encoded[4..8].copy_from_slice(b"NEW!");
    assert!(
        matches!(FontResource::decode(encoded, limits()), Err(DecodeError::UnsupportedFlavor(value)) if value == *b"NEW!")
    );
}

#[test]
fn unknown_collection_face_flavor_has_an_unsupported_error() {
    let mut directory = vec![63];
    directory.extend_from_slice(b"TEST");
    directory.push(0); // Empty opaque table.
    directory.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    directory.extend_from_slice(&[1, 1]); // One face, one table reference.
    directory.extend_from_slice(b"NEW!");
    directory.push(0);
    let encoded = wrap(&directory, 1, &[], *b"ttcf");
    assert!(
        matches!(FontResource::decode(encoded, limits()), Err(DecodeError::UnsupportedFlavor(value)) if value == *b"NEW!")
    );
}
