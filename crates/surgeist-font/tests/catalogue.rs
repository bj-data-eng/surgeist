use surgeist_font::{CatalogueError, DecodeLimits, FontCatalogue, FontResource, SourceFormat};

const AHEM: &[u8] = include_bytes!("fixtures/ahem-original/Ahem.woff2");
const CFF: &[u8] = include_bytes!("fixtures/source-code-pro/SourceCodePro-Regular.otf");

fn limits() -> DecodeLimits {
    DecodeLimits::new(1 << 20, 1 << 20, 1 << 20, 100, 10, 24 << 20, 32 << 20)
}

fn decode(bytes: &[u8]) -> FontResource {
    FontResource::decode(bytes.to_vec(), limits()).unwrap()
}

fn unnamed_face() -> Vec<u8> {
    // Structurally admissible SFNT containing one opaque table and no name.
    let mut bytes = vec![0; 32];
    bytes[..6].copy_from_slice(b"\0\x01\0\0\0\x01");
    bytes[12..16].copy_from_slice(b"TEST");
    bytes[20..24].copy_from_slice(&28u32.to_be_bytes());
    bytes[24..28].copy_from_slice(&4u32.to_be_bytes());
    bytes
}

// Lay out two real SFNT payloads (or a deliberately unnamed face) with
// separate directories. Table offsets in TTC records are file-relative.
fn collection(first: &[u8], second: &[u8]) -> FontResource {
    let mut bytes = b"ttcf\0\x01\0\0\0\0\0\x02\0\0\0\0\0\0\0\0".to_vec();
    for (index, face) in [first, second].into_iter().enumerate() {
        bytes.resize(bytes.len().next_multiple_of(4), 0);
        let base = u32::try_from(bytes.len()).unwrap();
        let record = 12 + 4 * index;
        bytes[record..record + 4].copy_from_slice(&base.to_be_bytes());
        bytes.extend_from_slice(face);
        let count = u16::from_be_bytes(face[4..6].try_into().unwrap());
        for table in 0..usize::from(count) {
            let relative = 20 + 16 * table;
            let offset = u32::from_be_bytes(face[relative..relative + 4].try_into().unwrap());
            let start = usize::try_from(base).unwrap() + relative;
            bytes[start..start + 4].copy_from_slice(&(base + offset).to_be_bytes());
        }
    }
    decode(&bytes)
}

#[test]
fn registered_only_starts_empty_with_a_distinct_issuer() {
    let mut first = FontCatalogue::registered_only();
    let second = FontCatalogue::registered_only();
    assert!(first.registered_faces().next().is_none());
    let initial = first.publication().clone();
    assert_eq!(initial.generation(), second.publication().generation());
    assert_ne!(&initial, second.publication());
    let refreshed = first.refresh().unwrap();
    assert_eq!(refreshed.generation(), initial.generation() + 1);
    assert_eq!(&refreshed, first.publication());
    assert_ne!(initial, refreshed);
    assert!(first.registered_faces().next().is_none());
}

#[test]
fn registration_and_refresh_keep_actual_resources_and_face_order() {
    let ahem = decode(AHEM);
    let mixed = collection(CFF, ahem.bytes());
    let mut catalogue = FontCatalogue::registered_only();
    let first = catalogue.register(ahem.clone()).unwrap();
    let second = catalogue.register(mixed.clone()).unwrap();
    assert_eq!(second.generation(), first.generation() + 1);
    assert_eq!(&second, catalogue.publication());
    let expected = vec![
        ahem.face(0).unwrap(),
        mixed.face(0).unwrap(),
        mixed.face(1).unwrap(),
    ];
    let faces: Vec<_> = catalogue.registered_faces().collect();
    assert_eq!(faces, expected);
    assert_eq!(faces[0].resource().source().format(), SourceFormat::Woff2);
    assert_eq!(faces[0].resource().source().bytes(), AHEM);
    assert_eq!(faces[0].resource().bytes().as_ptr(), ahem.bytes().as_ptr());
    assert_eq!(faces[1].sfnt_version(), *b"OTTO");
    assert_eq!(faces[2].sfnt_version(), [0, 1, 0, 0]);

    let third = catalogue.refresh().unwrap();
    assert_eq!(third.generation(), second.generation() + 1);
    assert_eq!(&third, catalogue.publication());
    assert_eq!(catalogue.registered_faces().collect::<Vec<_>>(), expected);
    drop(catalogue);
    drop(expected);
    drop(ahem);
    drop(mixed);
    assert_eq!(faces[0].resource().source().bytes(), AHEM);
    assert!(faces[1].table(*b"CFF ").is_some());
}

#[test]
fn refusal_of_one_collection_face_publishes_nothing() {
    let previous = decode(AHEM);
    let mut catalogue = FontCatalogue::registered_only();
    let publication = catalogue.register(previous.clone()).unwrap();
    let incomplete = collection(previous.bytes(), &unnamed_face());
    assert_eq!(incomplete.face_count(), 2);
    assert!(matches!(
        catalogue.register(incomplete),
        Err(CatalogueError::FacesNotAdmitted {
            expected: 2,
            admitted: 1
        })
    ));
    assert_eq!(catalogue.publication(), &publication);
    assert_eq!(
        catalogue.registered_faces().collect::<Vec<_>>(),
        vec![previous.face(0).unwrap()]
    );
}
