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

// Two independently representable SFNT encodings that trade coordinate bytes
// for repeated flags: short zero vectors share 0x33 with +1, while signed
// 16-bit vectors share 0x21 across +1,+256,-1,-256. Both stay within short loca.
fn alternate_widths(glyphs: usize, points: usize, long: bool, wide: bool) -> Vec<u8> {
    let source_span = (14 + 2 * points.div_ceil(256) + points * if wide { 2 } else { 1 } + 1) & !1;
    let original_length = source_span * glyphs;
    assert_eq!(original_length, if wide { 110444 } else { 88716 });
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
    for _ in 0..glyphs {
        for point in 0..points {
            outline.push(if wide {
                [11, 13, 10, 12][point % 4]
            } else {
                11
            });
        }
    }
    for _ in 0..glyphs {
        for point in 0..points {
            outline.push(u8::from(point % 2 == 0));
        }
        outline.push(0);
    }
    outline.resize(outline.len() + bbox, 0);
    let mut directory = vec![10];
    base128(original_length, &mut directory);
    base128(outline.len(), &mut directory);
    directory.push(11);
    base128((glyphs + 1) * if long { 4 } else { 2 }, &mut directory);
    base128(0, &mut directory);
    wrap(&directory, 2, &outline, [0, 1, 0, 0])
}

fn assert_alternate_points(
    resource: &FontResource,
    glyphs: usize,
    points: usize,
    long: bool,
    wide: bool,
) {
    let face = resource.face(0).unwrap();
    let glyf = face.table(*b"glyf").unwrap();
    let loca = face.table(*b"loca").unwrap();
    assert_eq!(loca.len(), (glyphs + 1) * if long { 4 } else { 2 });
    let at = |index: usize| {
        if long {
            u32::from_be_bytes(loca[index * 4..index * 4 + 4].try_into().unwrap()) as usize
        } else {
            usize::from(u16::from_be_bytes(
                loca[index * 2..index * 2 + 2].try_into().unwrap(),
            )) * 2
        }
    };
    assert_eq!(at(glyphs), glyf.len());
    for glyph in 0..glyphs {
        let data = &glyf[at(glyph)..at(glyph + 1)];
        assert_eq!(&data[..2], &[0, 1]);
        assert_eq!(
            usize::from(u16::from_be_bytes(data[10..12].try_into().unwrap())) + 1,
            points
        );
        assert_eq!(&data[12..14], &[0, 0]); // No instructions.
        let mut offset = 14;
        let mut flags = Vec::new();
        while flags.len() < points {
            let flag = data[offset];
            offset += 1;
            assert_eq!(flag & 1, 1);
            let repeats = if flag & 8 != 0 {
                let n = usize::from(data[offset]);
                offset += 1;
                n
            } else {
                0
            };
            flags.resize(flags.len() + repeats + 1, flag);
        }
        assert_eq!(flags.len(), points);
        for (short, same, horizontal) in [(2, 16, true), (4, 32, false)] {
            let mut coordinate = 0i32;
            for (point, flag) in flags.iter().enumerate() {
                let delta = if flag & short != 0 {
                    let value = i32::from(data[offset]);
                    offset += 1;
                    if flag & same != 0 { value } else { -value }
                } else if flag & same != 0 {
                    0
                } else {
                    let value = i16::from_be_bytes(data[offset..offset + 2].try_into().unwrap());
                    offset += 2;
                    i32::from(value)
                };
                coordinate += delta;
                let expected = if !horizontal {
                    0
                } else if wide {
                    [1, 257, 256, 0][point % 4]
                } else {
                    (point / 2 + 1) as i32
                };
                assert_eq!(coordinate, expected, "glyph {glyph} point {point}");
            }
        }
        assert!(data[offset..].iter().all(|byte| *byte == 0));
    }
}

#[test]
fn explicit_zero_coordinates_can_preserve_a_short_loca_flag_run() {
    let control = FontResource::decode(alternate_widths(2, 44000, true, false), limits()).unwrap();
    assert_alternate_points(&control, 2, 44000, true, false);
    let resource =
        FontResource::decode(alternate_widths(2, 44000, false, false), limits()).unwrap();
    assert!(resource.face(0).unwrap().table(*b"glyf").unwrap().len() <= 131070);
    assert_alternate_points(&resource, 2, 44000, false, false);
}

#[test]
fn signed_word_coordinates_can_preserve_a_short_loca_flag_run() {
    let control = FontResource::decode(alternate_widths(1, 55000, true, true), limits()).unwrap();
    assert_alternate_points(&control, 1, 55000, true, true);
    let resource = FontResource::decode(alternate_widths(1, 55000, false, true), limits()).unwrap();
    assert!(resource.face(0).unwrap().table(*b"glyf").unwrap().len() <= 131070);
    assert_alternate_points(&resource, 1, 55000, false, true);
}
