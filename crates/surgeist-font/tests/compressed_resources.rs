use surgeist_font::{DecodeError, DecodeLimits, FontFace, FontResource, LimitKind, SourceFormat};

fn limits() -> DecodeLimits {
    DecodeLimits::new(2 << 20, 4 << 20, 2 << 20, 1000, 20, 32 << 20, 40 << 20)
}

fn u16_at(data: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(data[at..at + 2].try_into().unwrap())
}
fn u32_at(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(data[at..at + 4].try_into().unwrap())
}
fn put32(data: &mut [u8], at: usize, value: usize) {
    data[at..at + 4].copy_from_slice(&u32::try_from(value).unwrap().to_be_bytes());
}
fn base128(mut value: usize, data: &mut Vec<u8>) {
    let mut digits = [0u8; 5];
    let mut index = 4;
    digits[index] = (value & 127) as u8;
    while value > 127 {
        value >>= 7;
        index -= 1;
        digits[index] = (value as u8 & 127) | 128;
    }
    data.extend_from_slice(&digits[index..]);
}

// Independent RFC 7932 sections 9.1-9.2 wire writer: standard window 16,
// uncompressed non-final metablocks, four length nibbles, then an empty final
// metablock. It does not call any decoder or compression-library encoder.
fn literal_brotli(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return vec![6];
    }
    let mut result = Vec::new();
    for (index, block) in data.chunks(65536).enumerate() {
        let shift = if index == 0 { 4 } else { 3 };
        let header = ((block.len() - 1) << shift) | (1 << (shift + 16));
        result.extend_from_slice(&(header as u32).to_le_bytes()[..3]);
        result.extend_from_slice(block);
    }
    result.push(3);
    result
}

// Package independently sourced real SFNT tables with null WOFF2 transforms.
// Expectations remain the original table bytes, not this decoder's output.
fn null_woff2(sfnt: &[u8]) -> Vec<u8> {
    let count = u16_at(sfnt, 4);
    let mut directory = Vec::new();
    let mut payload = Vec::new();
    for index in 0..usize::from(count) {
        let record = 12 + index * 16;
        let tag = &sfnt[record..record + 4];
        let offset = u32_at(sfnt, record + 8) as usize;
        let length = u32_at(sfnt, record + 12) as usize;
        directory.push(if tag == b"glyf" || tag == b"loca" {
            255
        } else {
            63
        });
        directory.extend_from_slice(tag);
        base128(length, &mut directory);
        payload.extend_from_slice(&sfnt[offset..offset + length]);
    }
    let compressed = literal_brotli(&payload);
    let mut result = vec![0; 48];
    result[..4].copy_from_slice(b"wOF2");
    result[4..8].copy_from_slice(&sfnt[..4]);
    result[12..14].copy_from_slice(&count.to_be_bytes());
    put32(&mut result, 16, sfnt.len());
    put32(&mut result, 20, compressed.len());
    result.extend_from_slice(&directory);
    result.extend_from_slice(&compressed);
    let length = result.len();
    put32(&mut result, 8, length);
    result
}

fn raw_woff1(sfnt: &[u8]) -> Vec<u8> {
    let count = usize::from(u16_at(sfnt, 4));
    let mut result = vec![0; 44 + count * 20];
    result[..4].copy_from_slice(b"wOFF");
    result[4..8].copy_from_slice(&sfnt[..4]);
    result[12..14].copy_from_slice(&(count as u16).to_be_bytes());
    let mut physical: Vec<usize> = (0..count).collect();
    physical.sort_unstable_by_key(|index| u32_at(sfnt, 12 + index * 16 + 8));
    let mut decoded_length = 12 + count * 16;
    for index in physical {
        let record = 12 + index * 16;
        let entry = 44 + index * 20;
        let offset = u32_at(sfnt, record + 8) as usize;
        let length = u32_at(sfnt, record + 12) as usize;
        let start = result.len();
        result[entry..entry + 4].copy_from_slice(&sfnt[record..record + 4]);
        put32(&mut result, entry + 4, start);
        put32(&mut result, entry + 8, length);
        put32(&mut result, entry + 12, length);
        result[entry + 16..entry + 20].copy_from_slice(&sfnt[record + 4..record + 8]);
        result.extend_from_slice(&sfnt[offset..offset + length]);
        while !result.len().is_multiple_of(4) {
            result.push(0);
        }
        decoded_length += (length + 3) & !3;
    }
    let length = result.len();
    put32(&mut result, 8, length);
    put32(&mut result, 16, decoded_length);
    result
}

fn shared_collection_woff2(truetype: &[u8], cff: &[u8]) -> Vec<u8> {
    let counts = [u16_at(truetype, 4), u16_at(cff, 4)];
    let mut directory = Vec::new();
    let mut payload = Vec::new();
    for sfnt in [truetype, cff] {
        let mut order: Vec<usize> = (0..usize::from(u16_at(sfnt, 4))).collect();
        // REC 2024 section 5.5 requires adjacent glyf/loca entries in a
        // collection even when both use the null transform.
        let position = |tag: &[u8; 4]| {
            order
                .iter()
                .position(|index| &sfnt[12 + index * 16..16 + index * 16] == tag)
        };
        if let (Some(glyf), Some(loca)) = (position(b"glyf"), position(b"loca")) {
            let loca = order.remove(loca);
            order.insert(glyf + 1, loca);
        }
        for index in order {
            let at = 12 + index * 16;
            let tag = &sfnt[at..at + 4];
            let start = u32_at(sfnt, at + 8) as usize;
            let length = u32_at(sfnt, at + 12) as usize;
            directory.push(if tag == b"glyf" || tag == b"loca" {
                255
            } else {
                63
            });
            directory.extend_from_slice(tag);
            base128(length, &mut directory);
            payload.extend_from_slice(&sfnt[start..start + length]);
        }
    }
    directory.extend_from_slice(&0x0002_0000u32.to_be_bytes());
    directory.push(3); // Three original face indices; all indices below 253.
    // Physical global directory puts TrueType first; original face order starts CFF.
    for (sfnt, first) in [(cff, counts[0]), (truetype, 0), (truetype, 0)] {
        let count = u16_at(sfnt, 4);
        assert!(count < 253 && first + count < 253);
        directory.push(count as u8);
        directory.extend_from_slice(&sfnt[..4]);
        directory.extend((first..first + count).map(|index| index as u8));
    }
    let compressed = literal_brotli(&payload);
    let mut result = vec![0; 48];
    result[..4].copy_from_slice(b"wOF2");
    result[4..8].copy_from_slice(b"ttcf");
    result[12..14].copy_from_slice(&(counts[0] + counts[1]).to_be_bytes());
    put32(&mut result, 20, compressed.len());
    result.extend_from_slice(&directory);
    result.extend_from_slice(&compressed);
    let length = result.len();
    put32(&mut result, 8, length);
    result
}

fn assert_original_tables(sfnt: &[u8], face: &FontFace) {
    let count = u16_at(sfnt, 4);
    assert_eq!(face.table_count(), count);
    for index in 0..usize::from(count) {
        let record = 12 + index * 16;
        let tag = sfnt[record..record + 4].try_into().unwrap();
        let offset = u32_at(sfnt, record + 8) as usize;
        let length = u32_at(sfnt, record + 12) as usize;
        let mut expected = sfnt[offset..offset + length].to_vec();
        let mut actual = face.table(tag).unwrap().to_vec();
        if tag == *b"head" {
            // New directory offsets legitimately change whole-font adjustment.
            expected[8..12].fill(0);
            actual[8..12].fill(0);
        }
        assert_eq!(actual, expected, "table {tag:?}");
    }
}

#[test]
fn retained_original_ahem_woff2_is_a_distinct_real_decoded_resource() {
    let encoded = include_bytes!("fixtures/ahem-original/Ahem.woff2");
    let resource = FontResource::decode(encoded.to_vec(), limits()).unwrap();
    assert_eq!(resource.source().format(), SourceFormat::Woff2);
    assert_eq!(resource.source().bytes(), encoded);
    assert_ne!(resource.bytes(), encoded);
    let face = resource.face(0).unwrap();
    assert_eq!(u16_at(face.table(*b"head").unwrap(), 18), 1000);
    let hhea = face.table(*b"hhea").unwrap();
    assert_eq!(i16::from_be_bytes(hhea[4..6].try_into().unwrap()), 800);
    assert_eq!(i16::from_be_bytes(hhea[6..8].try_into().unwrap()), -200);
    assert_eq!(i16::from_be_bytes(hhea[8..10].try_into().unwrap()), 0);
    assert!(face.table(*b"glyf").is_some());
    assert!(face.table(*b"loca").is_some());
    let later = include_bytes!("../../surgeist-render/tests/fixtures/fonts/ahem/Ahem.ttf");
    let later = FontResource::decode(later.to_vec(), limits()).unwrap();
    assert_ne!(resource.bytes(), later.bytes());
    assert_ne!(resource.identity(), later.identity());
    assert_ne!(resource.source().identity(), later.source().identity());
    let identity = resource.identity();
    drop(resource);
    assert_eq!(face.resource().identity(), identity);
}

#[test]
fn independent_cff_tables_survive_null_transform_woff2() {
    let original = include_bytes!("fixtures/source-code-pro/SourceCodePro-Regular.otf");
    let encoded = null_woff2(original);
    let decoded = FontResource::decode(encoded, limits()).unwrap();
    let face = decoded.face(0).unwrap();
    assert_eq!(face.sfnt_version(), *b"OTTO");
    assert!(face.table(*b"CFF ").is_some());
    assert!(face.table(*b"GSUB").is_some());
    assert_original_tables(original, &face);
}

#[test]
fn independent_variable_color_font_keeps_every_table() {
    let original = include_bytes!("fixtures/color-fonts/test_glyphs-glyf_colr_1_variable.ttf");
    let encoded = null_woff2(original);
    let decoded = FontResource::decode(encoded, limits()).unwrap();
    let face = decoded.face(0).unwrap();
    for tag in [*b"fvar", *b"gvar", *b"COLR", *b"CPAL"] {
        assert!(face.table(tag).is_some(), "missing {tag:?}");
    }
    assert_original_tables(original, &face);
}

#[test]
fn real_truetype_and_cff_fonts_decode_from_woff1_with_exact_table_contents() {
    let truetype: &[u8] =
        include_bytes!("../../surgeist-render/tests/fixtures/fonts/ahem/Ahem.ttf");
    let cff: &[u8] = include_bytes!("fixtures/source-code-pro/SourceCodePro-Regular.otf");
    for original in [truetype, cff] {
        let encoded = raw_woff1(original);
        let decoded = FontResource::decode(encoded.clone(), limits()).unwrap();
        assert_eq!(decoded.source().format(), SourceFormat::Woff);
        assert_eq!(decoded.source().bytes(), encoded);
        assert_original_tables(original, &decoded.face(0).unwrap());
    }
}

#[test]
fn collection_entry_order_and_true_shared_table_backing_survive_public_decode() {
    let truetype = include_bytes!("../../surgeist-render/tests/fixtures/fonts/ahem/Ahem.ttf");
    let cff = include_bytes!("fixtures/source-code-pro/SourceCodePro-Regular.otf");
    let encoded = shared_collection_woff2(truetype, cff);
    let resource = FontResource::decode(encoded, limits()).unwrap();
    assert_eq!(resource.face_count(), 3);
    let first = resource.face(0).unwrap();
    let second = resource.face(1).unwrap();
    let third = resource.face(2).unwrap();
    assert_eq!(first.sfnt_version(), *b"OTTO");
    assert_eq!(second.sfnt_version(), [0, 1, 0, 0]);
    assert_ne!(second, third);
    assert_original_tables(cff, &first);
    assert_original_tables(truetype, &second);
    assert_original_tables(truetype, &third);
    assert_eq!(
        second.table(*b"glyf").unwrap().as_ptr(),
        third.table(*b"glyf").unwrap().as_ptr()
    );
    assert_eq!(
        second.table(*b"hmtx").unwrap().as_ptr(),
        third.table(*b"hmtx").unwrap().as_ptr()
    );
    assert_eq!(&resource.bytes()[24..36], &[0; 12]); // Reconstructed TTC v2 DSIG fields.
}

#[test]
fn reserved_and_reference_size_fields_do_not_reject_valid_woff2() {
    let mut encoded = include_bytes!("fixtures/ahem-original/Ahem.woff2").to_vec();
    encoded[14..16].fill(255);
    encoded[16..20].fill(0);
    assert!(FontResource::decode(encoded, limits()).is_ok());
}

#[test]
fn invalid_metadata_expansion_claim_does_not_prevent_loading_valid_woff2() {
    // WOFF2 section 6 inherits WOFF1 section 7: invalid metadata is ignored.
    // Its decoded-length claim is content information, not a file range.
    let original = include_bytes!("fixtures/ahem-original/Ahem.woff2");
    let baseline = FontResource::decode(original.to_vec(), limits()).unwrap();
    for claimed in [0usize, u32::MAX as usize] {
        let mut encoded = original.to_vec();
        while !encoded.len().is_multiple_of(4) {
            encoded.push(0);
        }
        let metadata_start = encoded.len();
        encoded.push(255); // Intentionally not a valid Brotli/XML metadata stream.
        put32(&mut encoded, 28, metadata_start);
        put32(&mut encoded, 32, 1);
        put32(&mut encoded, 36, claimed);
        let length = encoded.len();
        put32(&mut encoded, 8, length);
        let decoded = FontResource::decode(encoded, limits()).unwrap();
        assert_eq!(decoded.bytes(), baseline.bytes());
    }
}

#[test]
fn compressed_source_and_working_storage_limits_reject_without_a_partial_resource() {
    let encoded = include_bytes!("fixtures/ahem-original/Ahem.woff2");
    let n = 64 << 20;
    for (policy, kind) in [
        (
            DecodeLimits::new(encoded.len() - 1, n, n, 1000, 20, n, n),
            LimitKind::EncodedBytes,
        ),
        (
            DecodeLimits::new(n, n, n, 1000, 20, 1024, n),
            LimitKind::WorkspaceBytes,
        ),
        (
            DecodeLimits::new(n, n, n, 1000, 20, n, 1024),
            LimitKind::LiveBytes,
        ),
    ] {
        assert!(
            matches!(FontResource::decode(encoded.to_vec(), policy), Err(DecodeError::LimitExceeded {kind: actual,..}) if actual == kind)
        );
    }
}
