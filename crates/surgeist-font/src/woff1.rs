//! WOFF 1.0 reconstruction under the 2012 Recommendation.
//!
//! The directory and fixed inflater state are charged as temporary workspace;
//! reconstructed bytes are charged separately. Metadata is never inflated.

use std::mem::size_of;

use miniz_oxide::inflate::{
    TINFLStatus,
    core::{DecompressorOxide, decompress, inflate_flags},
};

use super::{
    DecodeError, DecodeLimits, FontSource, InvalidKind, LimitKind, add, check, invalid, multiply,
};

#[derive(Clone, Copy)]
struct Entry {
    tag: [u8; 4],
    offset: usize,
    compressed: usize,
    original: usize,
    checksum: u32,
    directory_index: usize,
}

/// Reconstructs every packaged table. Later SFNT admission owns face creation.
pub(super) fn decode(source: &FontSource, limits: DecodeLimits) -> Result<Vec<u8>, DecodeError> {
    let input = source.bytes();
    let header = span(input, 0, 44)?;
    if &header[..4] != b"wOFF" || integer(input, 8)? != input.len() || short(input, 14)? != 0 {
        return Err(invalid(InvalidKind::WoffHeader, 0));
    }
    let flavor: [u8; 4] = header[4..8].try_into().expect("complete WOFF header");
    match &flavor {
        b"\0\x01\0\0" | b"OTTO" | b"true" | b"typ1" => {}
        _ => return Err(DecodeError::UnsupportedFlavor(flavor)),
    }
    let count = usize::from(short(input, 12)?);
    if count == 0 {
        return Err(invalid(InvalidKind::WoffHeader, 12));
    }
    check(LimitKind::Faces, 1, limits.faces())?;
    check(LimitKind::Tables, count, limits.tables())?;
    let search = search_fields(count)?;
    let directory_end = add(44, multiply(count, 20)?)?;
    span(input, 44, multiply(count, 20)?)?;
    let mut decoded_size = add(12, multiply(count, 16)?)?;
    let mut compressed = false;
    let mut previous = None;
    for index in 0..count {
        let entry = entry(input, index)?;
        if !valid_tag(entry.tag) {
            return Err(invalid(InvalidKind::TableTag, 44 + index * 20));
        }
        if previous.is_some_and(|tag| tag >= entry.tag) {
            return Err(invalid(InvalidKind::TableOrder, 44 + index * 20));
        }
        previous = Some(entry.tag);
        if entry.compressed > entry.original || !entry.offset.is_multiple_of(4) {
            return Err(invalid(InvalidKind::WoffDirectory, 44 + index * 20));
        }
        check(LimitKind::TableBytes, entry.original, limits.table_bytes())?;
        span(input, entry.offset, entry.compressed)?;
        decoded_size = add(decoded_size, padded(entry.original)?)?;
        compressed |= entry.compressed != entry.original;
    }
    if integer(input, 16)? != decoded_size {
        return Err(invalid(InvalidKind::WoffHeader, 16));
    }
    check(
        LimitKind::DecodedBytes,
        decoded_size,
        limits.decoded_bytes(),
    )?;
    let directory_bytes = multiply(count, size_of::<Entry>())?;
    // The inflater is stack-owned, but its complete fixed state is charged too.
    let workspace = add(
        directory_bytes,
        if compressed {
            size_of::<DecompressorOxide>()
        } else {
            0
        },
    )?;
    check(
        LimitKind::WorkspaceBytes,
        workspace,
        limits.workspace_bytes(),
    )?;
    check(
        LimitKind::LiveBytes,
        add(source.retained_bytes()?, add(decoded_size, workspace)?)?,
        limits.live_bytes(),
    )?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(count)
        .map_err(|_| DecodeError::AllocationFailed)?;
    for index in 0..count {
        entries.push(entry(input, index)?);
    }
    // Zero-byte tables precede nonempty tables at the same offset. No physical
    // payload order exists between empty spans, so directory order breaks ties.
    entries.sort_unstable_by_key(|entry| (entry.offset, entry.compressed, entry.directory_index));
    let mut cursor = directory_end;
    for entry in &entries {
        if entry.offset != cursor {
            return Err(invalid(InvalidKind::WoffDirectory, entry.offset));
        }
        let end = add(entry.offset, entry.compressed)?;
        let next = add(entry.offset, padded(entry.compressed)?)?;
        zeros(input, end, next)?;
        cursor = next;
    }
    optional_blocks(input, cursor)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(decoded_size)
        .map_err(|_| DecodeError::AllocationFailed)?;
    output.resize(decoded_size, 0);
    output[..4].copy_from_slice(&flavor);
    output[4..6].copy_from_slice(&short(input, 12)?.to_be_bytes());
    for (field, value) in [6, 8, 10].into_iter().zip(search) {
        output[field..field + 2].copy_from_slice(&value.to_be_bytes());
    }
    let mut decoded_cursor = add(12, multiply(count, 16)?)?;
    let mut head = None;
    let mut signature = false;
    for entry in &entries {
        let end = add(decoded_cursor, entry.original)?;
        let table = &mut output[decoded_cursor..end];
        let encoded = span(input, entry.offset, entry.compressed)?;
        if entry.compressed == entry.original {
            table.copy_from_slice(encoded);
        } else {
            inflate(encoded, table, entry.offset)?;
        }
        let is_head = entry.tag == *b"head";
        if (is_head && entry.original < 12) || checksum(table, is_head) != entry.checksum {
            return Err(invalid(InvalidKind::WoffChecksum, entry.offset));
        }
        if is_head {
            head = Some(decoded_cursor);
        }
        signature |= entry.tag == *b"DSIG";
        let record = 12 + entry.directory_index * 16;
        output[record..record + 4].copy_from_slice(&entry.tag);
        output[record + 4..record + 8].copy_from_slice(&entry.checksum.to_be_bytes());
        let offset = u32::try_from(decoded_cursor).map_err(|_| DecodeError::SizeOverflow)?;
        let length = u32::try_from(entry.original).map_err(|_| DecodeError::SizeOverflow)?;
        output[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
        output[record + 12..record + 16].copy_from_slice(&length.to_be_bytes());
        decoded_cursor = add(decoded_cursor, padded(entry.original)?)?;
    }
    if let Some(head) = head
        && checksum(&output, false) != 0xb1b0_afba
    {
        // Correcting an invalid original adjustment must not silently leave
        // a purported digital signature attached to modified font bytes.
        if signature {
            return Err(invalid(InvalidKind::WoffChecksum, head + 8));
        }
        output[head + 8..head + 12].fill(0);
        let adjustment = 0xb1b0_afbau32.wrapping_sub(checksum(&output, false));
        output[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    }
    Ok(output)
}

fn entry(input: &[u8], index: usize) -> Result<Entry, DecodeError> {
    let record = add(44, multiply(index, 20)?)?;
    let bytes = span(input, record, 20)?;
    Ok(Entry {
        tag: bytes[..4].try_into().expect("complete directory record"),
        offset: integer(input, record + 4)?,
        compressed: integer(input, record + 8)?,
        original: integer(input, record + 12)?,
        checksum: u32::from_be_bytes(bytes[16..20].try_into().expect("complete checksum")),
        directory_index: index,
    })
}

fn optional_blocks(input: &[u8], mut cursor: usize) -> Result<(), DecodeError> {
    let metadata = integer(input, 24)?;
    let metadata_length = integer(input, 28)?;
    let metadata_original = integer(input, 32)?;
    let private = integer(input, 36)?;
    let private_length = integer(input, 40)?;
    if metadata == 0 {
        if metadata_length != 0 || metadata_original != 0 {
            return Err(invalid(InvalidKind::WoffHeader, 24));
        }
    } else {
        if metadata != cursor {
            return Err(invalid(InvalidKind::WoffHeader, 24));
        }
        span(input, metadata, metadata_length)?;
        cursor = add(metadata, metadata_length)?;
        // Content, zlib validity, XML validity and claimed expansion are not
        // interpreted. Invalid metadata must not prevent loading the font.
    }
    if private == 0 {
        if private_length != 0 {
            return Err(invalid(InvalidKind::WoffHeader, 36));
        }
        // A final metadata block SHOULD have no padding; tolerate its optional
        // alignment padding, but never other trailing data.
        if metadata != 0 && input.len() == padded(cursor)? {
            zeros(input, cursor, input.len())?;
            cursor = input.len();
        }
    } else {
        let next = padded(cursor)?;
        zeros(input, cursor, next)?;
        if private != next {
            return Err(invalid(InvalidKind::WoffHeader, 36));
        }
        span(input, private, private_length)?;
        cursor = add(private, private_length)?;
    }
    if cursor != input.len() {
        return Err(invalid(InvalidKind::WoffHeader, cursor));
    }
    Ok(())
}

fn inflate(input: &[u8], output: &mut [u8], offset: usize) -> Result<(), DecodeError> {
    let mut state = DecompressorOxide::new();
    let flags = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
        | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
    let (status, consumed, written) = decompress(&mut state, input, output, 0, flags);
    if status != TINFLStatus::Done || consumed != input.len() || written != output.len() {
        return Err(invalid(InvalidKind::CompressedData, offset));
    }
    Ok(())
}

fn checksum(input: &[u8], head: bool) -> u32 {
    input
        .chunks(4)
        .enumerate()
        .fold(0u32, |sum, (index, bytes)| {
            if head && index == 2 {
                return sum;
            }
            let mut word = [0; 4];
            word[..bytes.len()].copy_from_slice(bytes);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
}

fn search_fields(count: usize) -> Result<[u16; 3], DecodeError> {
    let selector = count.ilog2();
    let range = multiply(1usize << selector, 16)?;
    let shift = multiply(count, 16)?
        .checked_sub(range)
        .ok_or(DecodeError::SizeOverflow)?;
    Ok([
        u16::try_from(range).map_err(|_| invalid(InvalidKind::WoffHeader, 12))?,
        u16::try_from(selector).map_err(|_| DecodeError::SizeOverflow)?,
        u16::try_from(shift).map_err(|_| invalid(InvalidKind::WoffHeader, 12))?,
    ])
}

fn valid_tag(tag: [u8; 4]) -> bool {
    let mut seen_space = false;
    let mut non_space = false;
    for byte in tag {
        if !(0x20..=0x7e).contains(&byte) {
            return false;
        }
        if byte == b' ' {
            seen_space = true;
        } else if seen_space {
            return false;
        } else {
            non_space = true;
        }
    }
    non_space
}
fn padded(length: usize) -> Result<usize, DecodeError> {
    Ok(add(length, 3)? & !3)
}
fn zeros(input: &[u8], start: usize, end: usize) -> Result<(), DecodeError> {
    let length = end.checked_sub(start).ok_or(DecodeError::SizeOverflow)?;
    if span(input, start, length)?.iter().any(|&byte| byte != 0) {
        return Err(invalid(InvalidKind::WoffDirectory, start));
    }
    Ok(())
}
fn span(input: &[u8], offset: usize, length: usize) -> Result<&[u8], DecodeError> {
    input
        .get(offset..add(offset, length)?)
        .ok_or_else(|| invalid(InvalidKind::Truncated, offset))
}
fn integer(input: &[u8], offset: usize) -> Result<usize, DecodeError> {
    let value = u32::from_be_bytes(span(input, offset, 4)?.try_into().expect("four-byte span"));
    usize::try_from(value).map_err(|_| DecodeError::SizeOverflow)
}
fn short(input: &[u8], offset: usize) -> Result<u16, DecodeError> {
    Ok(u16::from_be_bytes(
        span(input, offset, 2)?.try_into().expect("two-byte span"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FontResource, SourceFormat};

    // RFC1950 envelope, fixed-Huffman DEFLATE, sixty-four zero bytes;
    // Adler-32 is (64 << 16) | 1. No compressor is part of this oracle.
    const ZERO64: [u8; 12] = [0x78, 0x9c, 0x63, 0x60, 0xa0, 0x0c, 0, 0, 0, 0x40, 0, 1];

    fn limits() -> DecodeLimits {
        DecodeLimits::new(1 << 20, 1 << 20, 1 << 20, 100, 10, 1 << 20, 1 << 20)
    }
    fn put(input: &mut [u8], field: usize, value: u32) {
        input[field..field + 4].copy_from_slice(&value.to_be_bytes());
    }
    fn source(input: Vec<u8>) -> FontSource {
        FontSource::from_encoded(input, SourceFormat::Woff, limits()).unwrap()
    }

    fn single(encoded: &[u8], original: u32, checksum: u32) -> Vec<u8> {
        let encoded_padded = (encoded.len() + 3) & !3;
        let mut bytes = vec![0; 64 + encoded_padded];
        bytes[..4].copy_from_slice(b"wOFF");
        bytes[4..8].copy_from_slice(&[0, 1, 0, 0]);
        let length = u32::try_from(bytes.len()).unwrap();
        put(&mut bytes, 8, length);
        bytes[12..14].copy_from_slice(&1u16.to_be_bytes());
        put(&mut bytes, 16, 28 + ((original + 3) & !3));
        bytes[44..48].copy_from_slice(b"TEST");
        put(&mut bytes, 48, 64);
        put(&mut bytes, 52, u32::try_from(encoded.len()).unwrap());
        put(&mut bytes, 56, original);
        put(&mut bytes, 60, checksum);
        bytes[64..64 + encoded.len()].copy_from_slice(encoded);
        bytes
    }

    fn malformed(input: Vec<u8>) -> InvalidKind {
        match decode(&source(input), limits()).unwrap_err() {
            DecodeError::InvalidContainer(error) => error.kind(),
            error => panic!("expected malformed WOFF, got {error}"),
        }
    }

    #[test]
    fn literal_uncompressed_table_reconstructs_exact_sfnt_bytes() {
        let encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
        let backing = source(encoded.clone());
        let decoded = decode(&backing, limits()).unwrap();
        assert_eq!(
            decoded,
            [
                0, 1, 0, 0, 0, 1, 0, 16, 0, 0, 0, 0, b'T', b'E', b'S', b'T', 1, 2, 3, 4, 0, 0, 0,
                28, 0, 0, 0, 4, 1, 2, 3, 4,
            ]
        );
        let resource = FontResource::from_backing(backing, Some(decoded), limits()).unwrap();
        assert_eq!(resource.source().format(), SourceFormat::Woff);
        assert_eq!(resource.source().bytes(), encoded);
        assert_eq!(
            resource.face(0).unwrap().table(*b"TEST"),
            Some([1, 2, 3, 4].as_slice())
        );
    }

    #[test]
    fn literal_zlib_stream_reconstructs_exact_fixed_output() {
        let decoded = decode(&source(single(&ZERO64, 64, 0)), limits()).unwrap();
        assert_eq!(&decoded[28..], &[0; 64]);
        assert_eq!(decoded.len(), 92);
        let resource = FontResource::from_sfnt(decoded, limits()).unwrap();
        assert_eq!(
            resource.face(0).unwrap().table(*b"TEST"),
            Some([0; 64].as_slice())
        );
    }

    #[test]
    fn wrong_zlib_adler_truncation_trailing_bytes_and_expansion_lengths_reject() {
        let mut stream = ZERO64;
        stream[11] = 0;
        assert_eq!(
            malformed(single(&stream, 64, 0)),
            InvalidKind::CompressedData
        );
        assert_eq!(
            malformed(single(&ZERO64[..11], 64, 0)),
            InvalidKind::CompressedData
        );
        let mut trailing = ZERO64.to_vec();
        trailing.push(0);
        assert_eq!(
            malformed(single(&trailing, 64, 0)),
            InvalidKind::CompressedData
        );
        assert_eq!(
            malformed(single(&ZERO64, 63, 0)),
            InvalidKind::CompressedData
        );
        assert_eq!(
            malformed(single(&ZERO64, 65, 0)),
            InvalidKind::CompressedData
        );
        let mut no_header = ZERO64;
        no_header[0] = 0;
        assert_eq!(
            malformed(single(&no_header, 64, 0)),
            InvalidKind::CompressedData
        );
    }

    #[test]
    fn per_table_checksums_include_zero_padding_and_reject_mismatch() {
        let encoded = single(&[1, 2, 3], 3, 0x0102_0300);
        let decoded = decode(&source(encoded.clone()), limits()).unwrap();
        assert_eq!(&decoded[28..], &[1, 2, 3, 0]);
        let mut incorrect = encoded;
        put(&mut incorrect, 60, 0x0102_0301);
        assert_eq!(malformed(incorrect), InvalidKind::WoffChecksum);
    }

    #[test]
    fn cff_shaping_and_color_tables_keep_payload_and_original_physical_order() {
        let mut encoded = vec![0; 116];
        encoded[..4].copy_from_slice(b"wOFF");
        encoded[4..8].copy_from_slice(b"OTTO");
        put(&mut encoded, 8, 116);
        encoded[12..14].copy_from_slice(&3u16.to_be_bytes());
        put(&mut encoded, 16, 72);
        for (index, tag, offset, payload) in [
            (0, *b"CFF ", 112, [1, 2, 3, 4]),
            (1, *b"COLR", 108, [5, 6, 7, 8]),
            (2, *b"GSUB", 104, [9, 10, 11, 12]),
        ] {
            let record = 44 + index * 20;
            encoded[record..record + 4].copy_from_slice(&tag);
            put(&mut encoded, record + 4, offset);
            put(&mut encoded, record + 8, 4);
            put(&mut encoded, record + 12, 4);
            put(&mut encoded, record + 16, u32::from_be_bytes(payload));
            let offset = offset as usize;
            encoded[offset..offset + 4].copy_from_slice(&payload);
        }
        let decoded = decode(&source(encoded), limits()).unwrap();
        assert_eq!(&decoded[..12], b"OTTO\0\x03\0\x20\0\x01\0\x10");
        assert_eq!(&decoded[60..], &[9, 10, 11, 12, 5, 6, 7, 8, 1, 2, 3, 4]);
        for (field, offset) in [(20, 68u32), (36, 64), (52, 60)] {
            assert_eq!(&decoded[field..field + 4], offset.to_be_bytes());
        }
        let resource = FontResource::from_sfnt(decoded, limits()).unwrap();
        let face = resource.face(0).unwrap();
        assert_eq!(face.table(*b"CFF "), Some([1, 2, 3, 4].as_slice()));
        assert_eq!(face.table(*b"COLR"), Some([5, 6, 7, 8].as_slice()));
        assert_eq!(face.table(*b"GSUB"), Some([9, 10, 11, 12].as_slice()));
    }

    #[test]
    fn malformed_header_lengths_reserved_and_declared_sfnt_size_reject() {
        for (field, value) in [(8, 67), (16, 31), (16, 36)] {
            let mut encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
            put(&mut encoded, field, value);
            assert_eq!(malformed(encoded), InvalidKind::WoffHeader);
        }
        let mut encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
        encoded[15] = 1;
        assert_eq!(malformed(encoded), InvalidKind::WoffHeader);
        let mut encoded = single(&[], 0, 0);
        encoded[12..14].fill(0);
        assert_eq!(malformed(encoded), InvalidKind::WoffHeader);
    }

    #[test]
    fn unknown_sfnt_flavor_is_typed_unsupported_instead_of_malformed() {
        let mut encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
        encoded[4..8].copy_from_slice(b"NEW!");
        assert!(
            matches!(decode(&source(encoded), limits()), Err(DecodeError::UnsupportedFlavor(flavor)) if flavor == *b"NEW!")
        );
    }

    #[test]
    fn overlapping_gapped_out_of_range_and_extraneous_regions_reject() {
        for (field, value) in [(48, 44), (48, 68), (48, 65), (52, 5)] {
            let mut encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
            put(&mut encoded, field, value);
            assert!(decode(&source(encoded), limits()).is_err());
        }
        let mut gap = single(&[1, 2, 3, 4], 4, 0x0102_0304);
        gap.splice(64..64, [0; 4]);
        put(&mut gap, 8, 72);
        put(&mut gap, 48, 68);
        assert_eq!(malformed(gap), InvalidKind::WoffDirectory);
        let mut trailing = single(&[1, 2, 3, 4], 4, 0x0102_0304);
        trailing.extend([0; 4]);
        put(&mut trailing, 8, 72);
        assert_eq!(malformed(trailing), InvalidKind::WoffHeader);
        let mut padding = single(&[1, 2, 3], 3, 0x0102_0300);
        padding[67] = 1;
        assert_eq!(malformed(padding), InvalidKind::WoffDirectory);
    }

    fn optional(metadata: &[u8], private: &[u8]) -> Vec<u8> {
        let mut encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
        put(&mut encoded, 24, 68);
        put(&mut encoded, 28, u32::try_from(metadata.len()).unwrap());
        put(&mut encoded, 32, u32::MAX);
        encoded.extend_from_slice(metadata);
        if !private.is_empty() {
            while !encoded.len().is_multiple_of(4) {
                encoded.push(0);
            }
            let offset = u32::try_from(encoded.len()).unwrap();
            put(&mut encoded, 36, offset);
            put(&mut encoded, 40, u32::try_from(private.len()).unwrap());
            encoded.extend_from_slice(private);
        }
        let length = u32::try_from(encoded.len()).unwrap();
        put(&mut encoded, 8, length);
        encoded
    }

    #[test]
    fn ignored_metadata_and_arbitrary_private_contents_do_not_change_decoded_font() {
        let reference = decode(&source(single(&[1, 2, 3, 4], 4, 0x0102_0304)), limits()).unwrap();
        for encoded in [
            optional(b"not zlib or XML", &[]),
            optional(b"bad", &[0xff, 0, 0xff]),
            optional(&[], &[]),
        ] {
            assert_eq!(decode(&source(encoded), limits()).unwrap(), reference);
        }
        let encoded = source(optional(b"bad", &[0xff]));
        let tight = DecodeLimits::new(1024, 32, 4, 1, 1, 1024, 4096);
        assert_eq!(decode(&encoded, tight).unwrap(), reference);
    }

    #[test]
    fn optional_block_ranges_incoherent_absence_and_nonzero_alignment_reject() {
        for (field, value) in [(24, 64), (28, 100), (36, 68), (40, 100)] {
            let mut encoded = optional(b"bad", &[0xff]);
            put(&mut encoded, field, value);
            assert!(decode(&source(encoded), limits()).is_err());
        }
        let mut encoded = optional(b"bad", &[0xff]);
        encoded[71] = 1;
        assert_eq!(malformed(encoded), InvalidKind::WoffDirectory);
        for (field, value) in [(28, 1), (32, 1), (40, 1)] {
            let mut encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
            put(&mut encoded, field, value);
            assert_eq!(malformed(encoded), InvalidKind::WoffHeader);
        }
    }

    #[test]
    fn table_count_face_table_size_output_workspace_and_live_caps_are_enforced() {
        let n = 1 << 20;
        let input = source(single(&ZERO64, 64, 0));
        for (policy, expected) in [
            (DecodeLimits::new(n, n, n, 0, 1, n, n), LimitKind::Tables),
            (DecodeLimits::new(n, n, n, 1, 0, n, n), LimitKind::Faces),
            (
                DecodeLimits::new(n, n, 63, 1, 1, n, n),
                LimitKind::TableBytes,
            ),
            (
                DecodeLimits::new(n, 91, n, 1, 1, n, n),
                LimitKind::DecodedBytes,
            ),
            (
                DecodeLimits::new(n, n, n, 1, 1, 0, n),
                LimitKind::WorkspaceBytes,
            ),
            (
                DecodeLimits::new(n, n, n, 1, 1, n, 100),
                LimitKind::LiveBytes,
            ),
        ] {
            match decode(&input, policy).unwrap_err() {
                DecodeError::LimitExceeded {
                    kind,
                    required,
                    limit,
                } => {
                    assert_eq!(kind, expected);
                    assert!(required > limit);
                }
                other => panic!("expected limit error, got {other}"),
            }
        }
        let fixed_workspace = size_of::<Entry>() + size_of::<DecompressorOxide>();
        assert!(
            decode(
                &input,
                DecodeLimits::new(n, 92, 64, 1, 1, fixed_workspace, n)
            )
            .is_ok()
        );
        match decode(
            &input,
            DecodeLimits::new(n, 92, 64, 1, 1, fixed_workspace - 1, n),
        )
        .unwrap_err()
        {
            DecodeError::LimitExceeded {
                kind: LimitKind::WorkspaceBytes,
                ..
            } => {}
            other => panic!("expected fixed inflater workspace limit, got {other}"),
        }
    }

    fn head_font(adjustment: u32) -> Vec<u8> {
        let mut encoded = vec![0; 100];
        encoded[..4].copy_from_slice(b"wOFF");
        encoded[4..8].copy_from_slice(&[0, 1, 0, 0]);
        put(&mut encoded, 8, 100);
        encoded[12..14].copy_from_slice(&2u16.to_be_bytes());
        put(&mut encoded, 16, 60);
        encoded[44..48].copy_from_slice(b"TEST");
        put(&mut encoded, 48, 96);
        put(&mut encoded, 52, 4);
        put(&mut encoded, 56, 4);
        put(&mut encoded, 60, 0x0102_0304);
        encoded[64..68].copy_from_slice(b"head");
        put(&mut encoded, 68, 84);
        put(&mut encoded, 72, 12);
        put(&mut encoded, 76, 12);
        put(&mut encoded, 92, adjustment);
        encoded[96..].copy_from_slice(&[1, 2, 3, 4]);
        encoded
    }

    #[test]
    fn head_table_checksum_ignores_adjustment_and_whole_font_checksum_is_repaired() {
        let decoded = decode(&source(head_font(0)), limits()).unwrap();
        let total = decoded
            .chunks_exact(4)
            .map(|word| u32::from_be_bytes(word.try_into().unwrap()))
            .fold(0u32, u32::wrapping_add);
        assert_eq!(total, 0xb1b0_afba);
        // Directory checksum remains zero for head, because its adjustment word
        // is treated as zero, independently of the whole-font correction.
        assert_eq!(&decoded[32..36], &[0; 4]);
        let adjustment = u32::from_be_bytes(decoded[52..56].try_into().unwrap());
        assert_ne!(adjustment, 0);
        assert_eq!(
            decode(&source(head_font(adjustment)), limits()).unwrap(),
            decoded
        );
    }

    #[test]
    fn valid_signed_container_keeps_signature_and_invalid_adjustment_does_not_silently_modify_it() {
        let decoded = decode(&source(head_font(0)), limits()).unwrap();
        let adjustment = u32::from_be_bytes(decoded[52..56].try_into().unwrap())
            .wrapping_add(u32::from_be_bytes(*b"TEST"))
            .wrapping_sub(u32::from_be_bytes(*b"DSIG"));
        let mut signed = head_font(adjustment);
        signed[44..48].copy_from_slice(b"DSIG");
        let decoded = decode(&source(signed.clone()), limits()).unwrap();
        let resource = FontResource::from_sfnt(decoded, limits()).unwrap();
        assert_eq!(
            resource.face(0).unwrap().table(*b"DSIG"),
            Some([1, 2, 3, 4].as_slice())
        );
        put(&mut signed, 92, 0);
        assert_eq!(malformed(signed), InvalidKind::WoffChecksum);
    }

    #[test]
    fn duplicate_unsorted_and_invalid_tags_reject_before_reconstruction() {
        for tag in [*b"TEST", *b"AAAA"] {
            let mut encoded = head_font(0);
            encoded[64..68].copy_from_slice(&tag);
            assert_eq!(malformed(encoded), InvalidKind::TableOrder);
        }
        for tag in [*b"\0EST", *b"    ", *b"T ST"] {
            let mut encoded = single(&[1, 2, 3, 4], 4, 0x0102_0304);
            encoded[44..48].copy_from_slice(&tag);
            assert_eq!(malformed(encoded), InvalidKind::TableTag);
        }
    }
}
