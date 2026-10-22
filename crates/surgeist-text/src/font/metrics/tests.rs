use super::*;
use crate::{
    FontCatalogueMode, FontData, System, SystemOptions,
    test_fonts::{remove_tables, replace_tables, table, variable_ahem},
};

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn system() -> System {
    System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap()
}
fn registered(bytes: Vec<u8>) -> (System, FontData) {
    let mut system = system();
    system.register_fonts(bytes).unwrap();
    let data = system.registered_font_faces().next().unwrap().to_owned();
    (system, data)
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}
fn changed(tag: [u8; 4], offset: usize, value: i16) -> Vec<u8> {
    let mut bytes = table(AHEM, tag).to_vec();
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    replace_tables(AHEM, vec![(tag, bytes)])
}

#[test]
fn glyph_free_registered_ahem_metrics_have_literal_scaled_center_and_resource_lifetime() {
    let (system, data) = registered(AHEM.to_vec());
    let basis = system.font_catalogue_identity().clone();
    let stats = system.stats();
    let face = system.registered_font_faces().next().unwrap();
    assert_eq!(face.bytes(), AHEM);
    assert_eq!(face.index(), 0);
    assert_eq!(face.catalogue_identity(), &basis);
    assert!(face.is_same_face(&data.face()));
    // The result outlives the temporary instance, not its actual strong resource.
    let metrics = data
        .unsynthesized_instance(20.0, &[])
        .unwrap()
        .metrics(&basis)
        .unwrap();
    assert_eq!(metrics.instance().font_size(), 20.0);
    assert!(metrics.instance().face().is_same_face(&data.face()));
    assert_eq!(metrics.units_per_em(), 1000);
    assert_eq!(metrics.ascent(), 16.0);
    assert_eq!(metrics.descent(), -4.0);
    assert_eq!(metrics.leading(), 0.0);
    // Ahem OS/2 strike top259/size50 and head UPEM1000, not values read from result.
    close(metrics.strikeout().center_offset(), 4.68);
    assert_eq!(metrics.strikeout().thickness(), 1.0);
    close(
        30.0 - metrics.strikeout().center_offset() - metrics.strikeout().thickness() / 2.0,
        24.82,
    );
    assert_eq!(system.stats(), stats);
    drop(system);
    assert_eq!(data.bytes(), AHEM);
    assert_eq!(metrics.ascent(), 16.0);
    assert_eq!(
        data.unsynthesized_instance(20.0, &[])
            .unwrap()
            .metrics(&basis)
            .unwrap()
            .descent(),
        -4.0
    );
}

#[test]
fn real_missing_or_nonpositive_strike_derivatives_use_actual_size_fallback() {
    for bytes in [
        remove_tables(AHEM, &[*b"OS/2"]),
        changed(*b"OS/2", 26, 0),
        changed(*b"OS/2", 26, -1),
    ] {
        let (system, data) = registered(bytes);
        for (size, ascent, center, thickness) in [(18.0, 14.4, 7.2, 1.0), (36.0, 28.8, 14.4, 2.0)] {
            let metrics = data
                .unsynthesized_instance(size, &[])
                .unwrap()
                .metrics(system.font_catalogue_identity())
                .unwrap();
            close(metrics.ascent(), ascent);
            close(metrics.strikeout().center_offset(), center);
            assert_eq!(metrics.strikeout().thickness(), thickness);
        }
    }
}

#[test]
fn publication_qualification_is_separate_from_preserved_resource_equality() {
    let (mut system, old) = registered(AHEM.to_vec());
    let basis = old.catalogue_identity().clone();
    system.refresh_fonts().unwrap();
    let fresh = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(
        old, fresh,
        "same actual replayed Blob/index must remain resource-equal"
    );
    assert!(old.face().is_same_face(&fresh.face()));
    assert_ne!(old.catalogue_identity(), fresh.catalogue_identity());
    let stats = system.stats();
    let error = old
        .unsynthesized_instance(20.0, &[])
        .unwrap()
        .metrics(system.font_catalogue_identity())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FontCatalogueMismatch);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontCatalogueMismatch {
            expected: system.font_catalogue_identity().clone(),
            actual: basis.clone()
        })
    );
    assert_eq!(
        old.unsynthesized_instance(20.0, &[])
            .unwrap()
            .metrics(&basis)
            .unwrap()
            .ascent(),
        16.0
    );
    assert_eq!(
        fresh
            .unsynthesized_instance(20.0, &[])
            .unwrap()
            .metrics(system.font_catalogue_identity())
            .unwrap()
            .ascent(),
        16.0
    );
    let (foreign, _) = registered(AHEM.to_vec());
    assert_eq!(
        foreign.font_catalogue_identity().generation(),
        basis.generation()
    );
    assert_eq!(
        old.unsynthesized_instance(20.0, &[])
            .unwrap()
            .metrics(foreign.font_catalogue_identity())
            .unwrap_err()
            .code,
        ErrorCode::FontCatalogueMismatch
    );
    assert_eq!(system.stats(), stats);
    system.register_fonts(AHEM.to_vec()).unwrap();
    let faces: Vec<_> = system.registered_font_faces().collect();
    assert_eq!(faces.len(), 2);
    assert!(old.face().is_same_face(&faces[0]));
    assert!(!faces[0].is_same_face(&faces[1]));
    assert!(
        faces
            .iter()
            .all(|face| face.catalogue_identity() == system.font_catalogue_identity())
    );
    assert_eq!(old.bytes(), AHEM);
}

#[test]
fn registered_query_uses_request_then_real_collection_face_order_only() {
    let mut system = system();
    assert_eq!(system.registered_font_faces().count(), 0);
    let collection = crate::test_fonts::two_face_collection(AHEM.to_vec());
    system.register_fonts(collection.clone()).unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let faces: Vec<_> = system.registered_font_faces().collect();
    assert_eq!(
        faces.iter().map(|face| face.index()).collect::<Vec<_>>(),
        [0, 1, 0]
    );
    assert_eq!(faces[0].bytes(), collection);
    assert_eq!(faces[1].bytes(), collection);
    assert_eq!(faces[2].bytes(), AHEM);
    assert!(!faces[0].is_same_face(&faces[1]));
    assert!(
        faces
            .iter()
            .all(|face| face.catalogue_identity() == system.font_catalogue_identity())
    );
    for face in faces {
        let data = face.to_owned();
        assert_eq!(
            data.unsynthesized_instance(20.0, &[])
                .unwrap()
                .metrics(system.font_catalogue_identity())
                .unwrap()
                .ascent(),
            16.0
        );
    }
}

#[test]
fn static_f64_metrics_preserve_extreme_sizes_signed_offsets_and_negative_leading() {
    let (system, data) = registered(AHEM.to_vec());
    for size in [f32::from_bits(1), f32::MAX] {
        let metrics = data
            .unsynthesized_instance(size, &[])
            .unwrap()
            .metrics(system.font_catalogue_identity())
            .unwrap();
        assert_eq!(metrics.ascent(), 800.0 * (f64::from(size) / 1000.0));
        assert_eq!(metrics.descent(), -200.0 * (f64::from(size) / 1000.0));
        assert!(metrics.ascent().is_finite() && metrics.ascent() > 0.0);
        assert!(
            metrics.strikeout().thickness().is_finite() && metrics.strikeout().thickness() > 0.0
        );
    }
    for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(
            data.unsynthesized_instance(size, &[]).unwrap_err().code,
            ErrorCode::InvalidFontInstance
        );
    }
    let mut hhea = table(AHEM, *b"hhea").to_vec();
    hhea[8..10].copy_from_slice(&(-100i16).to_be_bytes());
    let mut os2 = table(AHEM, *b"OS/2").to_vec();
    os2[28..30].copy_from_slice(&(-100i16).to_be_bytes());
    let (system, data) = registered(replace_tables(
        AHEM,
        vec![(*b"hhea", hhea), (*b"OS/2", os2)],
    ));
    let metrics = data
        .unsynthesized_instance(20.0, &[])
        .unwrap()
        .metrics(system.font_catalogue_identity())
        .unwrap();
    assert_eq!(metrics.leading(), -2.0);
    assert_eq!(metrics.strikeout().center_offset(), -2.5);
}

#[test]
fn actual_variable_default_and_nondefault_without_mvar_preserve_global_metrics() {
    let (system, data) = registered(variable_ahem());
    for coordinates in [&[][..], &[8192][..]] {
        let instance = data.unsynthesized_instance(20.0, coordinates).unwrap();
        assert_eq!(instance.axis_count(), 1);
        assert_eq!(
            instance.normalized_coords().collect::<Vec<_>>(),
            if coordinates.is_empty() {
                vec![0]
            } else {
                vec![8192]
            }
        );
        let metrics = instance.metrics(system.font_catalogue_identity()).unwrap();
        assert_eq!(metrics.ascent(), 16.0);
        assert_eq!(metrics.descent(), -4.0);
        assert_eq!(metrics.leading(), 0.0);
        close(metrics.strikeout().center_offset(), 4.68);
        assert_eq!(metrics.strikeout().thickness(), 1.0);
    }
    let (system, data) = registered(replace_tables(AHEM, vec![(*b"MVAR", vec![0])]));
    assert_eq!(
        data.unsynthesized_instance(20.0, &[])
            .unwrap()
            .metrics(system.font_catalogue_identity())
            .unwrap_err()
            .detail(),
        Some(&ErrorDetail::FontMetricTable {
            table: Table::Mvar,
            reason: TableError::VariationWithoutAxes
        })
    );
}

#[test]
fn used_table_refusals_are_precise_without_claiming_earlier_registration_as_metric_work() {
    let mut bad_head = table(AHEM, *b"head").to_vec();
    bad_head[18..20].copy_from_slice(&0u16.to_be_bytes());
    let cases = [
        (
            replace_tables(AHEM, vec![(*b"head", bad_head)]),
            ErrorDetail::FontMetricTable {
                table: Table::Head,
                reason: TableError::InvalidUnitsPerEm { value: 0 },
            },
        ),
        (
            replace_tables(AHEM, vec![(*b"hhea", vec![0])]),
            ErrorDetail::FontMetricTable {
                table: Table::Hhea,
                reason: TableError::Malformed,
            },
        ),
        (
            replace_tables(AHEM, vec![(*b"OS/2", vec![0])]),
            ErrorDetail::FontMetricTable {
                table: Table::Os2,
                reason: TableError::Malformed,
            },
        ),
        (
            remove_tables(AHEM, &[*b"hhea", *b"OS/2"]),
            ErrorDetail::MissingFontLineMetrics,
        ),
    ];
    let authority = system();
    for (bytes, expected) in cases {
        let mut actual = system();
        let prior = actual.font_catalogue_identity().clone();
        let stats = actual.stats();
        match actual.register_fonts(bytes.clone()) {
            Ok(()) => {
                let data = actual.registered_font_faces().next().unwrap().to_owned();
                assert_eq!(
                    data.unsynthesized_instance(20.0, &[])
                        .unwrap()
                        .metrics(actual.font_catalogue_identity())
                        .unwrap_err()
                        .detail(),
                    Some(&expected)
                );
            }
            Err(error) => {
                // This is explicitly registration evidence, not executed metrics.
                assert_eq!(error.code, ErrorCode::FontLoadFailed);
                assert_eq!(actual.font_catalogue_identity(), &prior);
                assert_eq!(actual.stats(), stats);
            }
        }
        // Owner-private preflight covers the used-table branch even where public
        // registration rejects first. No public raw resource/token issuer exists.
        let candidate = FontData {
            data: bytes.into(),
            index: 0,
            catalogue: authority.font_catalogue_identity().clone(),
        };
        assert_eq!(
            candidate
                .unsynthesized_instance(20.0, &[])
                .unwrap()
                .metrics(authority.font_catalogue_identity())
                .unwrap_err()
                .detail(),
            Some(&expected)
        );
    }
    for which in [Table::Head, Table::Hhea, Table::Os2, Table::Mvar] {
        let tag = match which {
            Table::Head => *b"head",
            Table::Hhea => *b"hhea",
            Table::Os2 => *b"OS/2",
            Table::Mvar => *b"MVAR",
        };
        let base = if which == Table::Mvar {
            replace_tables(AHEM, vec![(*b"MVAR", vec![0])])
        } else {
            AHEM.to_vec()
        };
        let count = usize::from(u16::from_be_bytes(base[4..6].try_into().unwrap()));
        let record = (0..count)
            .map(|i| 12 + i * 16)
            .find(|&at| base[at..at + 4] == tag)
            .unwrap();
        let mut inaccessible = base.clone();
        inaccessible[record + 8..record + 12].copy_from_slice(&u32::MAX.to_be_bytes());
        let mut duplicate = base.clone();
        let other = if record == 12 { 28 } else { 12 };
        duplicate[other..other + 4].copy_from_slice(&tag);
        for (bytes, reason) in [
            (inaccessible, TableError::Inaccessible),
            (duplicate, TableError::Duplicate),
        ] {
            let candidate = FontData {
                data: bytes.into(),
                index: 0,
                catalogue: authority.font_catalogue_identity().clone(),
            };
            assert_eq!(
                candidate
                    .unsynthesized_instance(20.0, &[])
                    .unwrap()
                    .metrics(authority.font_catalogue_identity())
                    .unwrap_err()
                    .detail(),
                Some(&ErrorDetail::FontMetricTable {
                    table: which,
                    reason
                })
            );
        }
    }
}

#[test]
fn real_normalizer_handles_nonfinite_fallback_and_rejects_invalid_required_inputs() {
    for strike in [
        None,
        Some((f64::NAN, 1.0)),
        Some((1.0, f64::INFINITY)),
        Some((1.0, 0.0)),
        Some((1.0, -1.0)),
    ] {
        let result = normalize_strike(18.0, 14.4, strike).unwrap();
        close(result.center_offset(), 7.2);
        assert_eq!(result.thickness(), 1.0);
    }
    for (size, ascent) in [
        (0.0, 1.0),
        (-1.0, 1.0),
        (f64::NAN, 1.0),
        (18.0, -1.0),
        (18.0, f64::NAN),
    ] {
        assert!(matches!(
            normalize_strike(size, ascent, None).unwrap_err().detail(),
            Some(ErrorDetail::InvalidFontMetric { .. })
        ));
    }
    assert_eq!(
        normalize_strike(18.0, 0.0, None).unwrap().center_offset(),
        0.0
    );
    assert!(normalize_strike(18.0, 14.0, Some((-f64::MAX, f64::MAX))).is_err());
}
