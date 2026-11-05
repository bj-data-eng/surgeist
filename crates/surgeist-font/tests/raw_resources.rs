use std::hash::{DefaultHasher, Hash, Hasher};

use surgeist_font::{
    DecodeError, DecodeLimits, FontResource, InvalidKind, LimitKind, SourceFormat,
};

fn limits() -> DecodeLimits {
    DecodeLimits::new(1 << 20, 1 << 20, 1 << 20, 100, 10, 1 << 20, 1 << 20)
}

// Authored container fixtures; table payload bytes have no claimed font semantics.
fn sfnt() -> Vec<u8> {
    let mut bytes = vec![0; 32];
    bytes[..4].copy_from_slice(&[0, 1, 0, 0]);
    bytes[4..6].copy_from_slice(&1u16.to_be_bytes());
    bytes[6..8].copy_from_slice(&16u16.to_be_bytes());
    bytes[12..16].copy_from_slice(b"TEST");
    bytes[20..24].copy_from_slice(&28u32.to_be_bytes());
    bytes[24..28].copy_from_slice(&4u32.to_be_bytes());
    bytes[28..].copy_from_slice(&[1, 2, 3, 4]);
    bytes
}

fn collection() -> Vec<u8> {
    let mut bytes = vec![0; 80];
    bytes[..12].copy_from_slice(b"ttcf\0\x01\0\0\0\0\0\x02");
    // Collection order deliberately differs from physical directory order.
    bytes[12..16].copy_from_slice(&48u32.to_be_bytes());
    bytes[16..20].copy_from_slice(&20u32.to_be_bytes());
    for (offset, signature) in [(20, *b"OTTO"), (48, [0, 1, 0, 0])] {
        bytes[offset..offset + 28].copy_from_slice(&sfnt()[..28]);
        bytes[offset..offset + 4].copy_from_slice(&signature);
        bytes[offset + 20..offset + 24].copy_from_slice(&76u32.to_be_bytes());
    }
    bytes[76..80].copy_from_slice(&[5, 6, 7, 8]);
    bytes
}

fn invalid_kind(bytes: Vec<u8>) -> InvalidKind {
    match FontResource::from_sfnt(bytes, limits()).unwrap_err() {
        DecodeError::InvalidContainer(error) => error.kind(),
        error => panic!("expected structural error, got {error}"),
    }
}

fn bound(bytes: Vec<u8>, limits: DecodeLimits, expected: LimitKind) {
    match FontResource::from_sfnt(bytes, limits).unwrap_err() {
        DecodeError::LimitExceeded {
            kind,
            required,
            limit,
        } => {
            assert_eq!(kind, expected);
            assert!(required > limit);
        }
        error => panic!("expected limit error, got {error}"),
    }
}

#[test]
fn raw_payload_and_source_are_retained_without_copying() {
    let mut bytes = sfnt();
    bytes.extend([0xaa, 0xbb]); // Preserve exact source, including trailing bytes.
    let pointer = bytes.as_ptr();
    let resource = FontResource::from_sfnt(bytes, limits()).unwrap();
    assert_eq!(resource.bytes().as_ptr(), pointer);
    assert_eq!(resource.source().bytes().as_ptr(), pointer);
    assert_eq!(resource.source().format(), SourceFormat::Sfnt);
    assert_eq!(&resource.source().bytes()[32..], &[0xaa, 0xbb]);
    assert_eq!(resource.face_count(), 1);
    let face = resource.face(0).unwrap();
    assert_eq!(face.index(), 0);
    assert_eq!(face.table_count(), 1);
    assert_eq!(face.sfnt_version(), [0, 1, 0, 0]);
    assert_eq!(face.table(*b"TEST"), Some([1, 2, 3, 4].as_slice()));
    assert_eq!(face.table(*b"NONE"), None);
    assert!(resource.face(1).is_none());
    assert!(resource.face(u32::MAX).is_none());
}

#[test]
fn all_documented_raw_scaler_signatures_admit() {
    for signature in [[0, 1, 0, 0], *b"OTTO", *b"true", *b"typ1"] {
        let mut bytes = sfnt();
        bytes[..4].copy_from_slice(&signature);
        assert_eq!(
            FontResource::from_sfnt(bytes, limits())
                .unwrap()
                .face(0)
                .unwrap()
                .sfnt_version(),
            signature
        );
    }
    let mut bytes = sfnt();
    bytes[..4].copy_from_slice(b"wOFF");
    assert_eq!(invalid_kind(bytes), InvalidKind::Signature);
}

#[test]
fn identities_distinguish_equal_bytes_and_survive_cloning_and_resource_drop() {
    let first = FontResource::from_sfnt(sfnt(), limits()).unwrap();
    let second = FontResource::from_sfnt(sfnt(), limits()).unwrap();
    assert_ne!(first.identity(), second.identity());
    assert_ne!(first.source().identity(), second.source().identity());
    let clone = first.clone();
    assert_eq!(first.identity(), clone.identity());
    assert_eq!(first.source().identity(), clone.source().identity());
    let face = first.face(0).unwrap();
    let face_clone = face.clone();
    assert_eq!(face, face_clone);
    let hash = |value: &surgeist_font::FontFace| {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(hash(&face), hash(&face_clone));
    drop(first);
    drop(clone);
    assert_eq!(face.table(*b"TEST"), Some([1, 2, 3, 4].as_slice()));
}

#[test]
fn collection_preserves_index_order_and_accepts_identically_shared_tables() {
    let resource = FontResource::from_sfnt(collection(), limits()).unwrap();
    assert_eq!(resource.source().format(), SourceFormat::Collection);
    assert_eq!(resource.face_count(), 2);
    let first = resource.face(0).unwrap();
    let second = resource.face(1).unwrap();
    assert_eq!(first.sfnt_version(), [0, 1, 0, 0]);
    assert_eq!(second.sfnt_version(), *b"OTTO");
    assert_ne!(first, second);
    assert_eq!(first.resource().identity(), second.resource().identity());
    assert_eq!(first.table(*b"TEST"), Some([5, 6, 7, 8].as_slice()));
    assert_eq!(
        first.table(*b"TEST").unwrap().as_ptr(),
        second.table(*b"TEST").unwrap().as_ptr()
    );
}

#[test]
fn repeated_collection_directory_still_preserves_distinct_face_indices() {
    let mut bytes = collection();
    bytes[16..20].copy_from_slice(&48u32.to_be_bytes());
    let resource = FontResource::from_sfnt(bytes, limits()).unwrap();
    assert_eq!(resource.face_count(), 2);
    let first = resource.face(0).unwrap();
    let second = resource.face(1).unwrap();
    assert_ne!(first, second);
    assert_eq!(first.sfnt_version(), second.sfnt_version());
    assert_eq!(first.table(*b"TEST"), second.table(*b"TEST"));
}

#[test]
fn unknown_collection_versions_and_empty_collections_reject() {
    let mut bytes = collection();
    bytes[4..8].copy_from_slice(&0x0003_0000u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes), InvalidKind::CollectionVersion);
    let mut bytes = collection();
    bytes[8..12].fill(0);
    assert_eq!(invalid_kind(bytes), InvalidKind::EmptyCollection);
}

#[test]
fn later_collection_face_is_validated_before_resource_admission() {
    let mut bytes = collection();
    bytes[20 + 20..20 + 24].copy_from_slice(&80u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes), InvalidKind::Truncated);
    let mut bytes = collection();
    bytes[20..24].copy_from_slice(b"bad!");
    assert_eq!(invalid_kind(bytes), InvalidKind::Signature);
}

#[test]
fn truncated_headers_directories_and_table_ranges_reject() {
    for length in [0, 3, 4, 11, 12, 27, 31] {
        assert_eq!(
            invalid_kind(sfnt()[..length].to_vec()),
            InvalidKind::Truncated
        );
    }
    let mut bytes = sfnt();
    bytes[20..24].copy_from_slice(&0xffff_fffcu32.to_be_bytes());
    assert!(matches!(
        FontResource::from_sfnt(bytes, limits()),
        Err(DecodeError::InvalidContainer(_)) | Err(DecodeError::SizeOverflow)
    ));
    let mut bytes = collection();
    bytes.truncate(19);
    assert_eq!(invalid_kind(bytes), InvalidKind::Truncated);
}

#[test]
fn directory_header_and_partial_or_wrong_tag_table_overlaps_reject() {
    let mut bytes = sfnt();
    bytes[20..24].copy_from_slice(&12u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes), InvalidKind::Overlap);
    let mut bytes = collection();
    bytes[20 + 12..20 + 16].copy_from_slice(b"OTHR");
    assert_eq!(invalid_kind(bytes), InvalidKind::Overlap);
    let mut bytes = collection();
    bytes[20 + 24..20 + 28].copy_from_slice(&3u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes), InvalidKind::Overlap);
    let mut bytes = collection();
    bytes[12..16].copy_from_slice(&0u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes), InvalidKind::Signature);
}

#[test]
fn unique_sorted_printable_tags_and_aligned_table_offsets_are_required() {
    let mut bytes = sfnt();
    bytes.resize(52, 0);
    bytes[4..6].copy_from_slice(&2u16.to_be_bytes());
    bytes.copy_within(12..28, 28);
    bytes[20..24].copy_from_slice(&44u32.to_be_bytes());
    bytes[36..40].copy_from_slice(&48u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes.clone()), InvalidKind::TableOrder);
    bytes[28..32].copy_from_slice(b"AAAA");
    assert_eq!(invalid_kind(bytes), InvalidKind::TableOrder);
    for tag in [*b"\0EST", *b"    ", *b"T ST"] {
        let mut bytes = sfnt();
        bytes[12..16].copy_from_slice(&tag);
        assert_eq!(invalid_kind(bytes), InvalidKind::TableTag);
    }
    let mut bytes = sfnt();
    bytes[20..24].copy_from_slice(&27u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes), InvalidKind::UnalignedOffset);
}

#[test]
fn empty_table_is_present_and_derived_search_fields_are_not_trusted() {
    let mut bytes = sfnt();
    bytes[6..12].fill(0xff);
    bytes[24..28].fill(0);
    let resource = FontResource::from_sfnt(bytes, limits()).unwrap();
    assert_eq!(
        resource.face(0).unwrap().table(*b"TEST"),
        Some([].as_slice())
    );
}

#[test]
fn each_explicit_count_and_byte_limit_is_enforced() {
    let n = 1 << 20;
    bound(
        sfnt(),
        DecodeLimits::new(31, n, n, 100, 10, n, n),
        LimitKind::EncodedBytes,
    );
    bound(
        sfnt(),
        DecodeLimits::new(n, 31, n, 100, 10, n, n),
        LimitKind::DecodedBytes,
    );
    bound(
        sfnt(),
        DecodeLimits::new(n, n, 3, 100, 10, n, n),
        LimitKind::TableBytes,
    );
    bound(
        collection(),
        DecodeLimits::new(n, n, n, 1, 10, n, n),
        LimitKind::Tables,
    );
    bound(
        collection(),
        DecodeLimits::new(n, n, n, 100, 1, n, n),
        LimitKind::Faces,
    );
    bound(
        sfnt(),
        DecodeLimits::new(n, n, n, 100, 10, 0, n),
        LimitKind::WorkspaceBytes,
    );
    // Source ownership fits, but source + resource + validation workspace do not.
    bound(
        sfnt(),
        DecodeLimits::new(n, n, n, 100, 10, n, 180),
        LimitKind::LiveBytes,
    );
    assert!(FontResource::from_sfnt(sfnt(), DecodeLimits::new(32, 32, 4, 1, 1, n, n)).is_ok());
    let mut reserved = Vec::with_capacity(128);
    reserved.extend_from_slice(&sfnt());
    bound(
        reserved,
        DecodeLimits::new(127, n, n, 100, 10, n, n),
        LimitKind::EncodedBytes,
    );
    let mut reserved = Vec::with_capacity(1024);
    reserved.extend_from_slice(&sfnt());
    bound(
        reserved,
        DecodeLimits::new(n, n, n, 100, 10, n, 1000),
        LimitKind::LiveBytes,
    );
}

#[test]
fn version_two_collection_signature_region_is_structurally_validated() {
    let mut bytes = collection();
    // Add the v2 fields after the face-offset array and shift the rest by 12.
    bytes.splice(20..20, [0; 12]);
    bytes[4..8].copy_from_slice(&0x0002_0000u32.to_be_bytes());
    bytes[12..16].copy_from_slice(&60u32.to_be_bytes());
    bytes[16..20].copy_from_slice(&32u32.to_be_bytes());
    for offset in [32, 60] {
        bytes[offset + 20..offset + 24].copy_from_slice(&88u32.to_be_bytes());
    }
    assert_eq!(
        FontResource::from_sfnt(bytes.clone(), limits())
            .unwrap()
            .face_count(),
        2
    );
    bytes[20..24].copy_from_slice(b"DSIG");
    bytes[24..28].copy_from_slice(&4u32.to_be_bytes());
    bytes[28..32].copy_from_slice(&92u32.to_be_bytes());
    bytes.extend([9; 4]);
    assert!(FontResource::from_sfnt(bytes.clone(), limits()).is_ok());
    bytes[28..32].copy_from_slice(&88u32.to_be_bytes());
    assert_eq!(invalid_kind(bytes), InvalidKind::CollectionSignature);
}

#[test]
fn retained_real_ahem_font_admits_without_duplicating_the_fixture() {
    let bytes = include_bytes!("../../surgeist-render/tests/fixtures/fonts/ahem/Ahem.ttf");
    let resource = FontResource::from_sfnt(bytes.to_vec(), limits()).unwrap();
    assert_eq!(resource.face_count(), 1);
    let face = resource.face(0).unwrap();
    assert!(face.table(*b"head").is_some());
    assert!(face.table(*b"glyf").is_some());
    assert_eq!(resource.source().bytes(), bytes);
}
