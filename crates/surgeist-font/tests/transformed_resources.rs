use surgeist_font::{DecodeError, DecodeLimits, FontResource, InvalidKind};

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

#[test]
fn overlap_on_a_repeated_first_flag_preserves_short_loca() {
    // REC WOFF2 §5.1 stores overlap on the first physical TrueType flag
    // byte. The ordinary repeat count can still cover three identical
    // points: 14 header/end-point/instruction bytes + [0x79, 2] = 16.
    // Splitting off a singleton instead would take 18 bytes after alignment,
    // making 8,000 otherwise representable glyphs exceed short loca.
    let glyphs = 8000usize;
    let bbox = 4 * glyphs.div_ceil(32);
    let mut outline = vec![0, 0, 0, 1];
    outline.extend_from_slice(&(glyphs as u16).to_be_bytes());
    outline.extend_from_slice(&0u16.to_be_bytes());
    for length in [glyphs * 2, glyphs, glyphs * 3, glyphs * 4, 0, bbox, 0] {
        outline.extend_from_slice(&(length as u32).to_be_bytes());
    }
    for _ in 0..glyphs {
        outline.extend_from_slice(&[0, 1]);
    }
    outline.resize(outline.len() + glyphs, 3); // One contour of three points.
    outline.resize(outline.len() + glyphs * 7 + bbox, 0); // Flags, triplets, instructions, bbox map.
    outline.resize(outline.len() + glyphs.div_ceil(8), 255); // Overlap on every glyph.
    let mut directory = vec![10];
    base128(glyphs * 16, &mut directory);
    base128(outline.len(), &mut directory);
    directory.push(11);
    base128((glyphs + 1) * 2, &mut directory);
    base128(0, &mut directory);
    let encoded = wrap(&directory, 2, &outline, [0, 1, 0, 0]);
    let resource = FontResource::decode(encoded, limits()).unwrap();
    let face = resource.face(0).unwrap();
    let glyf = face.table(*b"glyf").unwrap();
    let loca = face.table(*b"loca").unwrap();
    assert_eq!(glyf.len(), glyphs * 16);
    assert_eq!(loca.len(), (glyphs + 1) * 2);
    let expected = [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0x79, 2];
    assert!(glyf.chunks_exact(16).all(|glyph| glyph == expected));
    for (index, word) in loca.chunks_exact(2).enumerate() {
        assert_eq!(
            usize::from(u16::from_be_bytes(word.try_into().unwrap())),
            index * 8
        );
    }
}

fn implicit_bbox_outline(first_y: u16) -> Vec<u8> {
    // Two on-curve points: (0, first_y) and (0, first_y + 1). Both delta
    // words fit; the inferred bounding box must also fit the SFNT i16 fields.
    let mut outline = vec![0, 0, 0, 0, 0, 1, 0, 0];
    for length in [2u32, 1, 2, 6, 0, 4, 0] {
        outline.extend_from_slice(&length.to_be_bytes());
    }
    outline.extend_from_slice(&[0, 1, 2, 127, 1, 0, 0]);
    outline.extend_from_slice(&first_y.to_be_bytes());
    outline.extend_from_slice(&[1, 0, 0, 0, 0, 0]); // Delta, instructions, bbox map.
    let mut directory = vec![10];
    base128(0, &mut directory); // Reconstructed glyf length is reference-only.
    base128(outline.len(), &mut directory);
    directory.extend_from_slice(&[11, 4, 0]);
    wrap(&directory, 2, &outline, [0, 1, 0, 0])
}

#[test]
fn inferred_glyph_bounds_must_fit_signed_sfnt_fields() {
    let control = FontResource::decode(implicit_bbox_outline(32766), limits()).unwrap();
    let face = control.face(0).unwrap();
    assert_eq!(
        &face.table(*b"glyf").unwrap()[8..10],
        &32767i16.to_be_bytes()
    );
    assert!(matches!(
        FontResource::decode(implicit_bbox_outline(32767), limits()),
        Err(DecodeError::InvalidContainer(error)) if error.kind() == InvalidKind::WoffGlyph
    ));
}

// One transformed simple glyph and short loca, with literal untransformed
// hhea/hmtx. Every declared face shares these tables but carries its own
// head/maxp fields. No hmtx reconstruction is needed to validate the join.
fn declared_outlines(faces: &[(u16, u16)]) -> Vec<u8> {
    let mut stream = vec![0, 0, 0, 0, 0, 1, 0, 0];
    for length in [2u32, 1, 1, 2, 0, 4, 0] {
        stream.extend_from_slice(&length.to_be_bytes());
    }
    stream.extend_from_slice(&[0, 1, 1, 23, 0, 0, 0, 0, 0, 0]);
    let mut hhea = [0; 36];
    hhea[34..].copy_from_slice(&1u16.to_be_bytes());
    stream.extend_from_slice(&hhea);
    stream.extend_from_slice(&[1, 244, 0, 1]); // Advance 500, bearing 1.
    let mut directory = vec![10, 18, 46, 11, 4, 0, 2, 36, 3, 4];
    for &(format, glyphs) in faces {
        let mut head = [0; 54];
        head[..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
        head[12..16].copy_from_slice(&0x5f0f_3cf5u32.to_be_bytes());
        head[18..20].copy_from_slice(&1000u16.to_be_bytes());
        head[50..52].copy_from_slice(&format.to_be_bytes());
        stream.extend_from_slice(&head);
        let mut maxp = [0; 32];
        maxp[..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
        maxp[4..6].copy_from_slice(&glyphs.to_be_bytes());
        stream.extend_from_slice(&maxp);
        directory.extend_from_slice(&[1, 54, 4, 32]);
    }
    let flavor = if faces.len() == 1 {
        [0, 1, 0, 0]
    } else {
        directory.extend_from_slice(&0x0001_0000u32.to_be_bytes());
        directory.push(faces.len() as u8);
        for index in 0..faces.len() {
            let head = (4 + 2 * index) as u8;
            directory.extend_from_slice(&[6, 0, 1, 0, 0, 0, 1, 2, 3, head, head + 1]);
        }
        *b"ttcf"
    };
    wrap(&directory, 4 + 2 * faces.len(), &stream, flavor)
}

#[test]
fn transformed_loca_must_match_the_literal_head_format() {
    let control = FontResource::decode(declared_outlines(&[(0, 1)]), limits()).unwrap();
    let face = control.face(0).unwrap();
    assert_eq!(face.table(*b"loca"), Some([0, 0, 0, 9].as_slice()));
    assert_eq!(face.table(*b"hmtx"), Some([1, 244, 0, 1].as_slice()));
    assert!(
        matches!(FontResource::decode(declared_outlines(&[(1, 1)]), limits()), Err(DecodeError::InvalidContainer(error)) if error.kind() == InvalidKind::WoffTransform)
    );
}

#[test]
fn transformed_glyph_count_must_match_literal_maxp() {
    assert!(
        matches!(FontResource::decode(declared_outlines(&[(0, 2)]), limits()), Err(DecodeError::InvalidContainer(error)) if error.kind() == InvalidKind::WoffTransform)
    );
}

#[test]
fn every_face_sharing_transformed_outlines_must_match_the_pair() {
    let control = FontResource::decode(declared_outlines(&[(0, 1), (0, 1)]), limits()).unwrap();
    assert_eq!(control.face_count(), 2);
    assert_eq!(
        control.face(0).unwrap().table(*b"loca").unwrap().as_ptr(),
        control.face(1).unwrap().table(*b"loca").unwrap().as_ptr()
    );
    assert!(
        matches!(FontResource::decode(declared_outlines(&[(0, 1), (1, 1)]), limits()), Err(DecodeError::InvalidContainer(error)) if error.kind() == InvalidKind::WoffTransform)
    );
}

#[test]
fn malformed_second_contour_reports_a_byte_offset() {
    let mut outline = vec![0, 0, 0, 0, 0, 2, 0, 0];
    for length in [4u32, 0, 0, 0, 0, 4, 0] {
        outline.extend_from_slice(&length.to_be_bytes());
    }
    outline.extend_from_slice(&[0, 0, 0xff, 0xfe, 0, 0, 0, 0]); // Empty, then invalid -2 contour word.
    let encoded = wrap(&[10, 0, 44, 11, 6, 0], 2, &outline, [0, 1, 0, 0]);
    let error = FontResource::decode(encoded, limits()).unwrap_err();
    assert!(
        matches!(error, DecodeError::InvalidContainer(ref detail) if detail.kind() == InvalidKind::WoffGlyph && detail.offset() == 2),
        "unexpected diagnostic: {error}"
    );
}

#[test]
fn unrepresentable_second_triplet_reports_its_byte_offset() {
    let mut outline = vec![0, 0, 0, 0, 0, 1, 0, 0];
    for length in [2u32, 1, 2, 9, 0, 4, 0] {
        outline.extend_from_slice(&length.to_be_bytes());
    }
    outline.extend_from_slice(&[0, 1, 2, 127, 127]);
    // First four-byte triplet is (0,0). Second has x=32768, which cannot be
    // emitted as a signed TrueType word. The substream byte position is 4.
    outline.extend_from_slice(&[0, 0, 0, 0, 0x80, 0, 0, 0, 0, 0, 0, 0, 0]);
    let mut directory = vec![10, 0];
    base128(outline.len(), &mut directory);
    directory.extend_from_slice(&[11, 4, 0]);
    let encoded = wrap(&directory, 2, &outline, [0, 1, 0, 0]);
    let error = FontResource::decode(encoded, limits()).unwrap_err();
    assert!(
        matches!(error, DecodeError::InvalidContainer(ref detail) if detail.kind() == InvalidKind::WoffGlyph && detail.offset() == 4),
        "unexpected diagnostic: {error}"
    );
}
