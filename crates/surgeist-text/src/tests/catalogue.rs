use super::*;

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn registered_only() -> System {
    System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .expect("registered-only catalogue initializes without system discovery")
}

fn ahem_layout(system: &mut System, text: &str) -> Layout {
    system
        .layout(
            Source::new(text),
            Style {
                font: Font::new().family("Ahem"),
                size: 20.0,
                ..Style::default()
            },
            Options {
                quantize: false,
                ..Options::default()
            },
        )
        .expect("the admitted Ahem font shapes the literal fixture")
}

fn assert_ahem_glyphs(layout: &Layout) {
    let runs = layout.glyph_runs();
    let glyphs: Vec<_> = runs.iter().flat_map(|run| run.glyphs()).collect();
    assert_eq!(
        glyphs.iter().map(|glyph| glyph.id()).collect::<Vec<_>>(),
        [35, 58]
    );
    assert_eq!(
        glyphs
            .iter()
            .map(|glyph| glyph.advance())
            .collect::<Vec<_>>(),
        [20.0, 20.0]
    );
    assert_eq!(
        glyphs.iter().map(|glyph| glyph.range()).collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_eq!(layout.metrics().full_width(), 40.0);
    for run in &runs {
        let data = run
            .font()
            .data()
            .expect("the actual font bytes are retained");
        assert_eq!(data.bytes(), AHEM);
        assert_eq!(data.index(), 0);
    }
}

#[test]
fn registered_font_drives_literal_glyphs_advances_and_family_fallback() {
    let mut system = registered_only();
    system
        .register_fonts(AHEM.to_vec())
        .expect("Ahem registers");
    assert_ahem_glyphs(&ahem_layout(&mut system, "AX"));
    let fallback = system
        .layout(
            Source::new("AX"),
            Style {
                font: Font::new().family("MissingFixtureFamily").family("Ahem"),
                size: 20.0,
                ..Style::default()
            },
            Options {
                quantize: false,
                ..Options::default()
            },
        )
        .expect("ordered fallback reaches the actual registered family");
    assert_ahem_glyphs(&fallback);
    assert_eq!(ahem_layout(&mut system, " ").metrics().full_width(), 20.0);
}

#[test]
fn catalogue_identity_distinguishes_issuers_and_publications_and_invalidates_cache() {
    let mut first = registered_only();
    let second = registered_only();
    let initial = first.font_catalogue_identity().clone();
    assert_eq!(
        initial.generation(),
        second.font_catalogue_identity().generation()
    );
    assert_ne!(&initial, second.font_catalogue_identity());
    assert_eq!(initial, initial.clone());

    first.register_fonts(AHEM.to_vec()).expect("Ahem registers");
    let registered = first.font_catalogue_identity().clone();
    assert_ne!(initial, registered);
    assert_eq!(registered.generation().get(), 1);
    let old = ahem_layout(&mut first, "AX");
    let same = ahem_layout(&mut first, "AX");
    assert_eq!(old.key(), same.key());
    assert_eq!(old.key().font_catalogue(), &registered);
    let stats = first.stats();
    assert_eq!(stats.layout_hits(), 1);

    first
        .refresh_fonts()
        .expect("refresh replays the actual registered bytes");
    let refreshed = first.font_catalogue_identity().clone();
    assert_ne!(refreshed, registered);
    assert_eq!(refreshed.generation().get(), 2);
    assert_eq!(first.stats().invalidations(), stats.invalidations() + 1);
    assert_eq!(first.stats().font_refreshes(), stats.font_refreshes() + 1);
    let fresh = ahem_layout(&mut first, "AX");
    assert_ne!(fresh.key(), old.key());
    assert_eq!(fresh.key().font_catalogue(), &refreshed);
    assert_eq!(first.stats().layout_misses(), stats.layout_misses() + 1);
    assert_ahem_glyphs(&fresh);
    assert_ahem_glyphs(&old);
    let hit = ahem_layout(&mut first, "AX");
    assert_eq!(fresh.key(), hit.key());
    assert_eq!(first.stats().layout_hits(), stats.layout_hits() + 1);
}

fn corrupt_charmap(mut font: Vec<u8>) -> Vec<u8> {
    let tables = usize::from(u16::from_be_bytes([font[4], font[5]]));
    let record = (0..tables)
        .map(|index| 12 + index * 16)
        .find(|offset| &font[*offset..*offset + 4] == b"cmap")
        .expect("the actual Ahem fixture has a cmap directory entry");
    font[record..record + 4].copy_from_slice(b"zzzz");
    font
}

fn two_face_collection(second: Vec<u8>) -> Vec<u8> {
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

fn invalid_collection_face() -> Vec<u8> {
    let mut bytes = two_face_collection(AHEM.to_vec());
    bytes[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    bytes
}

#[test]
fn invalid_or_partially_admitted_font_file_preserves_live_identity_cache_and_stats() {
    let mut system = registered_only();
    system
        .register_fonts(AHEM.to_vec())
        .expect("initial Ahem registers");
    let old = ahem_layout(&mut system, "AX");
    let identity = system.font_catalogue_identity().clone();
    let stats = system.stats();

    for (bytes, detail) in [
        (Vec::new(), ErrorDetail::InvalidFontContainer),
        (b"not a font".to_vec(), ErrorDetail::InvalidFontContainer),
        (AHEM[..8].to_vec(), ErrorDetail::InvalidFontContainer),
        (
            invalid_collection_face(),
            ErrorDetail::InvalidFontFace { index: 1 },
        ),
        (
            corrupt_charmap(AHEM.to_vec()),
            ErrorDetail::FontFacesNotAdmitted {
                expected: 1,
                admitted: 0,
            },
        ),
        (
            two_face_collection(corrupt_charmap(AHEM.to_vec())),
            ErrorDetail::FontFacesNotAdmitted {
                expected: 2,
                admitted: 1,
            },
        ),
    ] {
        let error = system
            .register_fonts(bytes)
            .expect_err("the whole font file is refused");
        assert_eq!(error.code, ErrorCode::FontLoadFailed);
        assert_eq!(error.detail(), Some(&detail));
        assert_eq!(system.font_catalogue_identity(), &identity);
        assert_eq!(system.stats(), stats);
        assert_ahem_glyphs(&old);
    }

    let reused = ahem_layout(&mut system, "AX");
    assert_eq!(reused.key(), old.key());
    assert_eq!(system.stats().layout_hits(), stats.layout_hits() + 1);
    assert_eq!(system.stats().layout_misses(), stats.layout_misses());
}

#[test]
fn font_bytes_and_historical_layout_survive_registration_refresh_and_system_disposal() {
    let mut system = registered_only();
    let input = AHEM.to_vec();
    system.register_fonts(input).expect("owned input registers");
    let layout = ahem_layout(&mut system, "AX");
    let old_key = layout.key();
    let retained = layout
        .glyph_runs()
        .into_iter()
        .next()
        .expect("Ahem run is present");
    system
        .register_fonts(AHEM.to_vec())
        .expect("equal bytes are another registration");
    assert_ne!(system.font_catalogue_identity(), old_key.font_catalogue());
    system
        .refresh_fonts()
        .expect("all actual registrations replay");
    drop(system);
    assert_eq!(layout.key(), old_key);
    assert_ahem_glyphs(&layout);
    drop(layout);
    let data = retained
        .font()
        .data()
        .expect("run owns the actual resource");
    assert_eq!(data.bytes(), AHEM);
    assert_eq!(data.index(), 0);
    assert_eq!(
        retained
            .glyphs()
            .iter()
            .map(|glyph| glyph.id())
            .collect::<Vec<_>>(),
        [35, 58]
    );
}

#[test]
fn registered_font_cursor_and_selection_use_the_same_literal_geometry() {
    let mut system = registered_only();
    system
        .register_fonts(AHEM.to_vec())
        .expect("Ahem registers");
    let layout = ahem_layout(&mut system, "AX");
    let cursor = layout.cursor(Cursor::new(1, Affinity::After));
    assert_eq!(cursor.rect().origin().x(), 20.0);
    let selection = layout.selection(Selection::new(
        Cursor::new(0, Affinity::After),
        Cursor::new(1, Affinity::Before),
    ));
    assert_eq!(selection.rects().len(), 1);
    assert_eq!(selection.rects()[0].rect().origin().x(), 0.0);
    assert_eq!(selection.rects()[0].rect().size().width(), 20.0);
}

#[cfg(feature = "text-accessibility")]
#[test]
fn registered_font_accessibility_retains_historical_layout_positions_after_refresh() {
    let mut system = registered_only();
    system
        .register_fonts(AHEM.to_vec())
        .expect("Ahem registers");
    let layout = ahem_layout(&mut system, "AX");
    let identity = layout.key();
    system.refresh_fonts().expect("catalogue refresh succeeds");
    drop(system);
    let mut state = Accessibility::new(10);
    let parent = accesskit::NodeId(1);
    let update = layout.accessibility_update(&mut state, parent, parent, Point::default());
    assert!(update.nodes.iter().any(|(id, _)| *id == parent));
    let selection = layout
        .access_selection(
            &state,
            Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(1, Affinity::Before),
            ),
        )
        .expect("actual historical grapheme positions are projected");
    assert_eq!(selection.anchor.character_index, 0);
    assert_eq!(selection.focus.character_index, 1);
    assert_eq!(layout.key(), identity);
    assert_ahem_glyphs(&layout);
}
