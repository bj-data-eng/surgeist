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
    let ahem = catalogue.family_by_name("Ahem").unwrap();
    assert!(catalogue.family_by_name("Source Code Pro").is_none());
    let incomplete = collection(CFF, &unnamed_face());
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
    assert!(catalogue.family_by_name("Source Code Pro").is_none());
    let selected = first_face(&mut catalogue, ahem);
    assert_eq!(selected, previous.face(0).unwrap());
    assert_eq!(
        selected.resource().bytes().as_ptr(),
        previous.bytes().as_ptr()
    );
}

#[test]
fn a_candidate_reuses_the_exact_registered_backing() {
    use std::ops::ControlFlow;

    use surgeist_font::FontAttributes;

    let ahem = decode(AHEM);
    let mut catalogue = FontCatalogue::registered_only();
    catalogue.register(ahem.clone()).unwrap();
    let family = catalogue.family_by_name("Ahem").unwrap();
    let publication = catalogue.publication().clone();

    let mut calls = 0;
    let outcome = catalogue
        .query()
        .visit_candidates(
            &[family.clone(), family],
            FontAttributes::default(),
            |font| {
                calls += 1;
                ControlFlow::Break(font.clone())
            },
        )
        .unwrap();
    let ControlFlow::Break(candidate) = outcome else {
        panic!("registered Ahem should produce a candidate");
    };
    assert_eq!(calls, 1);
    assert_eq!(candidate.publication(), &publication);
    assert_eq!(candidate.face(), &ahem.face(0).unwrap());
    assert_eq!(
        candidate.face().resource().bytes().as_ptr(),
        ahem.bytes().as_ptr()
    );
    assert_eq!(candidate.charmap().unwrap().glyph_id('\u{10ffff}'), None);
    assert_ne!(candidate.charmap().unwrap().glyph_id('A').unwrap(), 0);
    assert!(!candidate.synthesis().embolden());

    catalogue.refresh().unwrap();
    drop(catalogue);
    assert_eq!(candidate.face().resource().source().bytes(), AHEM);
}

#[test]
fn stale_and_foreign_families_are_rejected_before_visiting_any_candidate() {
    use std::ops::ControlFlow;

    use surgeist_font::FontAttributes;

    let mut catalogue = FontCatalogue::registered_only();
    let mut other = FontCatalogue::registered_only();
    let resource = decode(AHEM);
    catalogue.register(resource.clone()).unwrap();
    other.register(resource).unwrap();
    let stale = catalogue.family_by_name("Ahem").unwrap();
    let foreign = other.family_by_name("Ahem").unwrap();
    assert_eq!(
        stale.publication().generation(),
        foreign.publication().generation()
    );
    catalogue.refresh().unwrap();
    let current = catalogue.family_by_name("Ahem").unwrap();
    let mut query = catalogue.query();
    for (family, expected) in [
        (stale, CatalogueError::StaleFamily),
        (foreign, CatalogueError::ForeignFamily),
    ] {
        let outcome = query.visit_candidates(
            &[current.clone(), family],
            FontAttributes::default(),
            |_| -> ControlFlow<()> {
                panic!("all handles must be checked before any visit");
            },
        );
        assert_eq!(outcome.unwrap_err(), expected);
    }
}

#[test]
fn unrelated_names_produce_no_candidate() {
    use std::ops::ControlFlow;

    use surgeist_font::FontAttributes;

    let mut catalogue = FontCatalogue::registered_only();
    catalogue.register(decode(AHEM)).unwrap();
    assert!(catalogue.family_by_name("UnregisteredFamily").is_none());
    assert_eq!(catalogue.family_names().collect::<Vec<_>>(), ["Ahem"]);
    let outcome = catalogue
        .query()
        .visit_candidates(&[], FontAttributes::default(), |_| -> ControlFlow<()> {
            panic!("a registered font does not become an implicit generic fallback");
        })
        .unwrap();
    assert_eq!(outcome, ControlFlow::Continue(()));
}

#[test]
fn candidate_order_preserves_best_then_default_for_each_requested_family() {
    use std::ops::ControlFlow;

    use parlance::{FontStyle, FontWeight, FontWidth};
    use surgeist_font::FontAttributes;

    let regular = decode(AHEM);
    let mut bold_bytes = regular.bytes().to_vec();
    let table_count = u16::from_be_bytes(bold_bytes[4..6].try_into().unwrap());
    let os2 = bold_bytes[12..12 + 16 * usize::from(table_count)]
        .chunks_exact(16)
        .find(|record| &record[..4] == b"OS/2")
        .unwrap();
    let offset = usize::try_from(u32::from_be_bytes(os2[8..12].try_into().unwrap())).unwrap();
    // OpenType OS/2 places usWeightClass at byte 4. Keep the name and
    // all other attributes, and give the second fixture a bold weight.
    bold_bytes[offset + 4..offset + 6].copy_from_slice(&700u16.to_be_bytes());
    let bold = decode(&bold_bytes);
    let cff = decode(CFF);
    let mut catalogue = FontCatalogue::registered_only();
    for resource in [&regular, &bold, &cff] {
        catalogue.register(resource.clone()).unwrap();
    }
    let families = [
        catalogue.family_by_name("Ahem").unwrap(),
        catalogue.family_by_name("Source Code Pro").unwrap(),
    ];
    let mut faces = Vec::new();
    let result = catalogue
        .query()
        .visit_candidates(
            &families,
            FontAttributes::new(FontWidth::default(), FontStyle::Normal, FontWeight::BOLD),
            |font| {
                faces.push(font.face().clone());
                ControlFlow::<()>::Continue(())
            },
        )
        .unwrap();
    assert_eq!(result, ControlFlow::Continue(()));
    assert_eq!(
        faces,
        [
            bold.face(0).unwrap(),
            regular.face(0).unwrap(),
            cff.face(0).unwrap()
        ]
    );
}

#[test]
fn candidates_keep_the_collection_index_across_refresh() {
    let ahem = decode(AHEM);
    let resource = collection(CFF, ahem.bytes());
    let expected = resource.face(1).unwrap();
    let mut catalogue = FontCatalogue::registered_only();
    catalogue.register(resource.clone()).unwrap();
    for _ in 0..2 {
        let family = catalogue.family_by_name("ahem").unwrap();
        assert_eq!(catalogue.family_name(&family).unwrap(), "Ahem");
        let selected = first_face(&mut catalogue, family);
        assert_eq!(selected, expected);
        assert_eq!(selected.index(), 1);
        assert_eq!(
            selected.resource().bytes().as_ptr(),
            resource.bytes().as_ptr()
        );
        catalogue.refresh().unwrap();
    }
}

fn first_face(
    catalogue: &mut FontCatalogue,
    family: surgeist_font::FontFamily,
) -> surgeist_font::FontFace {
    use std::ops::ControlFlow;

    use surgeist_font::FontAttributes;

    let result = catalogue
        .query()
        .visit_candidates(&[family], FontAttributes::default(), |font| {
            ControlFlow::Break(font.face().clone())
        })
        .unwrap();
    let ControlFlow::Break(face) = result else {
        panic!("registered family must have a candidate");
    };
    face
}
