//! Independent used-reference observations from real registered font tables.
use crate::*;
use crate::{baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical};
use skrifa::{MetadataProvider, raw::TableProvider};

#[path = "upright_fixture.rs"]
mod fixture;

fn registered(bytes: Vec<u8>) -> (System, FontData) {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let data = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(data.bytes(), bytes);
    assert_eq!(data.index(), 0);
    (system, data)
}
fn script() -> FontBaselineScript {
    FontBaselineScript::from_iso15924(*b"Latn").unwrap()
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}
fn geometry(value: &UprightFontMetrics<'_>, expected: [f64; 5]) {
    for (actual, expected) in [
        value.em_under(),
        value.em_over(),
        value.central_coordinate_or_distance(),
        value.used_under(),
        value.used_over(),
    ]
    .into_iter()
    .zip(expected)
    {
        close(actual, expected);
    }
}
fn table_error(error: Error, table: FontMetricTable, reason: FontMetricTableError) {
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontMetricTable { table, reason })
    );
}

#[test]
fn unscaled_upright_facts_keep_actual_reference_instance_and_fractional_coordinates() {
    for (bytes, coordinates, expected, reference) in [
        (
            fixture::native(0x0001_1000, 700, -300, 200),
            vec![],
            [-300.0, 700.0, 200.0, -500.0, 500.0],
            UprightMetricReference::NominalVerticalPenX,
        ),
        (
            fixture::absolute(&vertical::native()),
            vec![],
            [0.0, 1200.0, 600.0, -600.0, 600.0],
            UprightMetricReference::FontDesignX,
        ),
        (
            fixture::fractional_vertical(),
            vec![8192, 8192],
            [0.0, 1000.75, 500.375, -500.375, 500.375],
            UprightMetricReference::FontDesignX,
        ),
    ] {
        let font = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(font.head().unwrap().units_per_em(), 1000);
        let (system, data) = registered(bytes);
        for size in [f32::from_bits(1), 10.0, 20.0, f32::MAX] {
            let instance = data.unsynthesized_instance(size, &coordinates).unwrap();
            let raw = instance
                .unscaled_upright_metrics(system.font_catalogue_identity(), script())
                .unwrap();
            assert_eq!(
                [
                    raw.em_under(),
                    raw.em_over(),
                    raw.central_coordinate_or_distance(),
                    raw.used_under(),
                    raw.used_over()
                ],
                expected
            );
            assert_eq!(raw.reference(), reference);
            assert_eq!(raw.units_per_em(), 1000);
            assert_eq!(raw.script(), script());
            assert!(raw.instance().face().is_same_face(&data.face()));
            assert_eq!(raw.instance().font_size(), size);
            assert_eq!(
                raw.instance().normalized_coords().collect::<Vec<_>>(),
                coordinates
            );
            let public = instance
                .upright_metrics(system.font_catalogue_identity(), script())
                .unwrap();
            geometry(&public, expected.map(|v| v * (f64::from(size) / 1000.0)));
            assert_eq!(raw.provenance(), public.provenance());
            assert_eq!(raw.base_script(), public.base_script());
            assert_eq!(raw.base_absence(), public.base_absence());
        }
    }
}

#[test]
fn upright_gap_is_signed_glyph_free_and_independently_demanded_even_with_base() {
    for gap in [200i16, -200, -1200] {
        let bytes = fixture::absolute(&fixture::native(0x0001_1000, 700, -300, gap));
        assert_eq!(
            i16::from_be_bytes(sfnt::table(&bytes, *b"vhea")[8..10].try_into().unwrap()),
            gap
        );
        // Font admission needs its real cmap. Actual nominal glyph access still
        // fails for both glyph zero and SPACE, independently of gap demand.
        let bytes = sfnt::remove_tables(&bytes, &[*b"vmtx", *b"glyf"]);
        let font = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(font.charmap().map(0x20_u32).unwrap().to_u32(), 3);
        let (system, data) = registered(bytes);
        let stats = system.stats();
        for size in [10.0, 20.0] {
            let instance = data.unsynthesized_instance(size, &[]).unwrap();
            assert_eq!(
                instance
                    .upright_line_gap(system.font_catalogue_identity())
                    .unwrap(),
                f64::from(gap)
            );
            let raw = instance
                .unscaled_upright_metrics(system.font_catalogue_identity(), script())
                .unwrap();
            assert_eq!([raw.used_under(), raw.used_over()], [-600.0, 600.0]);
            for glyph in [0, 3] {
                assert_eq!(
                    instance
                        .vertical_glyph_metrics(glyph, system.font_catalogue_identity())
                        .unwrap_err()
                        .detail(),
                    Some(&ErrorDetail::FontOutline {
                        glyph,
                        reason: FontOutlineError::UnavailableOutline,
                    }),
                );
            }
        }
        assert_eq!(system.stats(), stats);
    }
    let good = fixture::absolute(&fixture::native(0x0001_1000, 700, -300, 200));
    assert_eq!(
        u32::from_be_bytes(sfnt::table(&good, *b"vhea")[..4].try_into().unwrap()),
        0x0001_1000,
    );
    for (tag, which, reason) in [
        (
            *b"vhea",
            FontMetricTable::Vhea,
            FontMetricTableError::Malformed,
        ),
        (
            *b"MVAR",
            FontMetricTable::Mvar,
            FontMetricTableError::VariationWithoutAxes,
        ),
    ] {
        let (system, data) = registered(sfnt::replace_tables(&good, vec![(tag, vec![0])]));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        // An edge-only demand models the explicit-height boundary: BASE wins.
        assert!(
            instance
                .unscaled_upright_metrics(system.font_catalogue_identity(), script())
                .is_ok()
        );
        table_error(
            instance
                .upright_line_gap(system.font_catalogue_identity())
                .unwrap_err(),
            which,
            reason,
        );
    }
}

#[test]
fn absent_and_qualified_vhea_one_gap_ignore_reserved_values_and_undemanded_mvar() {
    for bytes in [
        vertical::absent(),
        fixture::native(0x0001_0000, 1200, -300, 900),
    ] {
        let bytes = sfnt::replace_tables(&bytes, vec![(*b"MVAR", vec![0])]);
        let (system, data) = registered(bytes);
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_eq!(
            instance
                .upright_line_gap(system.font_catalogue_identity())
                .unwrap(),
            0.0
        );
    }
}

#[test]
fn upright_gap_resolves_actual_instance_vlgp_with_existing_whole_unit_rounding() {
    for (records, expected) in [
        (vec![], [42.0, 42.0, 42.0]),
        (vec![(*b"vlgp", 100)], [42.0, 92.0, 142.0]),
        (vec![(*b"vlgp", 3)], [42.0, 44.0, 45.0]),
        (vec![(*b"vlgp", -3)], [42.0, 41.0, 39.0]),
    ] {
        let bytes = fixture::variable(&records);
        let font = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(font.fvar().unwrap().axis_count(), 1);
        assert_eq!(
            font.mvar().unwrap().value_record_count(),
            u16::try_from(records.len()).unwrap()
        );
        let (system, data) = registered(bytes);
        for (coord, expected) in [0, 8192, 16384].into_iter().zip(expected) {
            let coordinates = [coord];
            let instance = data.unsynthesized_instance(10.0, &coordinates).unwrap();
            assert_eq!(
                instance
                    .upright_line_gap(system.font_catalogue_identity())
                    .unwrap(),
                expected
            );
            let raw = instance
                .unscaled_upright_metrics(system.font_catalogue_identity(), script())
                .unwrap();
            assert_eq!([raw.used_under(), raw.used_over()], [-500.0, 500.0]);
        }
    }
}

#[test]
fn demanded_gap_keeps_strict_header_and_whole_record_mvar_failures() {
    let good = fixture::absolute(&fixture::native(0x0001_1000, 700, -300, 200));
    let header = sfnt::table(&good, *b"vhea").to_vec();
    let mut bad_version = header.clone();
    bad_version[..4].copy_from_slice(&0x0002_0000u32.to_be_bytes());
    let mut bad_format = header.clone();
    bad_format[32..34].copy_from_slice(&1i16.to_be_bytes());
    for (vhea, reason) in [
        (header[..35].to_vec(), FontMetricTableError::Malformed),
        (bad_version, FontMetricTableError::UnsupportedVersion),
        (bad_format, FontMetricTableError::Malformed),
    ] {
        let (system, data) = registered(sfnt::replace_tables(&good, vec![(*b"vhea", vhea)]));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert!(
            instance
                .unscaled_upright_metrics(system.font_catalogue_identity(), script())
                .is_ok()
        );
        table_error(
            instance
                .upright_line_gap(system.font_catalogue_identity())
                .unwrap_err(),
            FontMetricTable::Vhea,
            reason,
        );
    }
    let mut bad_row = fixture::mvar(&[(*b"stro", 100)]);
    bad_row[18..20].copy_from_slice(&99u16.to_be_bytes());
    let bytes = fixture::absolute(&sfnt::replace_tables(
        &fixture::variable(&[]),
        vec![(*b"MVAR", bad_row)],
    ));
    let (system, data) = registered(bytes);
    let instance = data.unsynthesized_instance(10.0, &[0]).unwrap();
    assert!(
        instance
            .unscaled_upright_metrics(system.font_catalogue_identity(), script())
            .is_ok()
    );
    table_error(
        instance
            .upright_line_gap(system.font_catalogue_identity())
            .unwrap_err(),
        FontMetricTable::Mvar,
        FontMetricTableError::VariationDeltaIndex {
            record: 0,
            outer: 0,
            inner: 99,
        },
    );
}

#[test]
fn actual_base_coordinates_precede_other_tables_and_scale_once_with_lifetime() {
    let bytes = fixture::absolute(&vertical::horizontal(800, -200, 0, 1001));
    let (system, data) = registered(bytes.clone());
    let basis = system.font_catalogue_identity().clone();
    let stats = system.stats();
    let small = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .upright_metrics(&basis, script())
        .unwrap();
    assert_eq!(small.reference(), UprightMetricReference::FontDesignX);
    assert_eq!(
        small.provenance(),
        UprightMetricProvenance::RawBaseline(FontBaselineProvenance::BaseEdgesCssMidpoint)
    );
    assert_eq!(small.base_script(), Some(*b"latn"));
    assert_eq!(small.base_absence(), None);
    geometry(&small, [0.0, 12.0, 6.0, -6.0, 6.0]);
    assert_eq!(small.units_per_em(), 1000);
    assert_eq!(small.script(), script());
    assert!(small.instance().face().is_same_face(&data.face()));
    assert_eq!(small.instance().font_size(), 10.0);
    let big = data
        .unsynthesized_instance(20.0, &[])
        .unwrap()
        .upright_metrics(&basis, script())
        .unwrap();
    geometry(&big, [0.0, 24.0, 12.0, -12.0, 12.0]);
    let nominal = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .vertical_glyph_metrics(vertical::X, &basis)
        .unwrap();
    close(nominal.origin_x(), 5.005);
    let old_raw = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .baseline(
            &basis,
            script(),
            FontBaselineAxis::VerticalX,
            FontBaseline::Central,
        )
        .unwrap();
    let FontBaselineOutcome::Available(raw) = old_raw.outcome() else {
        panic!("real authored BASE is available")
    };
    close(raw.coordinate(), 6.0);
    assert_eq!(system.stats(), stats);
    drop(system);
    assert_eq!(data.bytes(), bytes);
    geometry(&small, [0.0, 12.0, 6.0, -6.0, 6.0]);
    assert_eq!(small.instance().face().catalogue_identity(), &basis);

    let ignored = sfnt::replace_tables(
        &bytes,
        vec![
            (*b"vhea", vec![0]),
            (*b"MVAR", vec![0]),
            (*b"glyf", vec![0]),
        ],
    );
    let (system, data) = registered(ignored);
    let value = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .upright_metrics(system.font_catalogue_identity(), script())
        .unwrap();
    geometry(&value, [0.0, 12.0, 6.0, -6.0, 6.0]);
}

#[test]
fn real_cff_format_one_base_observation_keeps_nominal_outline_refusal_separate() {
    let bytes = fixture::absolute(&vertical::cff());
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert!(font.cff().is_ok());
    assert!(font.glyf().is_err());
    assert!(font.base().is_ok());
    let (system, data) = registered(bytes);
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    let value = instance
        .upright_metrics(system.font_catalogue_identity(), script())
        .unwrap();
    geometry(&value, [0.0, 12.0, 6.0, -6.0, 6.0]);
    assert_eq!(value.reference(), UprightMetricReference::FontDesignX);
    assert_eq!(
        value.provenance(),
        UprightMetricProvenance::RawBaseline(FontBaselineProvenance::BaseEdgesCssMidpoint)
    );
    let error = instance
        .vertical_glyph_metrics(1, system.font_catalogue_identity())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FontOutlineRefused);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontOutline {
            glyph: 1,
            reason: FontOutlineError::Unsupported {
                capability: FontOutlineCapability::PostScript,
            },
        })
    );
}

#[test]
fn fractional_vertical_base_remains_observable_before_provider_precision_admission() {
    let bytes = fixture::fractional_vertical();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.fvar().unwrap().axis_count(), 2);
    assert!(font.base().unwrap().item_var_store().is_some());
    let (system, data) = registered(bytes);
    let coordinates = [8192, 8192];
    for (size, top, center) in [(10.0, 10.0075, 5.00375), (1000.0, 1000.75, 500.375)] {
        let instance = data.unsynthesized_instance(size, &coordinates).unwrap();
        let value = instance
            .upright_metrics(system.font_catalogue_identity(), script())
            .unwrap();
        geometry(&value, [0.0, top, center, -center, center]);
        assert_eq!(value.reference(), UprightMetricReference::FontDesignX);
        assert_eq!(
            value.provenance(),
            UprightMetricProvenance::RawBaseline(FontBaselineProvenance::BaseEdgesCssMidpoint)
        );
        assert_eq!(
            value.instance().normalized_coords().collect::<Vec<_>>(),
            coordinates
        );
        assert_eq!(value.base_script(), Some(*b"latn"));
        let old = instance
            .baseline(
                system.font_catalogue_identity(),
                script(),
                FontBaselineAxis::VerticalX,
                FontBaseline::Central,
            )
            .unwrap();
        let FontBaselineOutcome::Available(raw) = old.outcome() else {
            panic!("fractional raw BASE stays available")
        };
        close(raw.coordinate(), center);
        if size == 1000.0 {
            assert_eq!(
                surgeist_shaper::VerticalOriginReference::try_font_design_x(
                    value.central_coordinate_or_distance()
                ),
                Err(surgeist_shaper::VerticalOriginReferenceError::NotHalfUnit)
            );
        }
        let error = instance
            .vertical_glyph_metrics(vertical::X, system.font_catalogue_identity())
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    }
}

#[test]
fn native_vhea_distances_keep_asymmetry_symmetry_signed_centers_and_gap_independence() {
    for (over, under, center, extent) in [
        (700, -300, 2.0, 5.0),
        (500, -500, 0.0, 5.0),
        (100, -900, -4.0, 5.0),
    ] {
        for gap in [0, 700] {
            let bytes = fixture::native(0x0001_1000, over, under, gap);
            let font = skrifa::FontRef::new(&bytes).unwrap();
            assert_eq!(font.vhea().unwrap().version().to_major_minor(), (1, 1));
            let (system, data) = registered(bytes);
            let value = data
                .unsynthesized_instance(10.0, &[])
                .unwrap()
                .upright_metrics(system.font_catalogue_identity(), script())
                .unwrap();
            geometry(
                &value,
                [
                    f64::from(under) / 100.0,
                    f64::from(over) / 100.0,
                    center,
                    -extent,
                    extent,
                ],
            );
            assert_eq!(
                value.reference(),
                UprightMetricReference::NominalVerticalPenX
            );
            assert_eq!(value.provenance(), UprightMetricProvenance::VheaTypographic);
            assert_eq!(value.base_absence(), Some(FontBaselineAbsence::BaseTable));
            assert_eq!(value.base_script(), None);
        }
    }
}

#[test]
fn native_cross_metrics_ignore_horizontal_ad_strikeout_and_unrequested_mvar_values() {
    let font = fixture::variable(&[
        (*b"hasc", 1000),
        (*b"hdsc", -1000),
        (*b"stro", 5000),
        (*b"vasc", 3),
        (*b"vdsc", -3),
        (*b"vlgp", 1000),
    ]);
    let mut hhea = sfnt::table(&font, *b"hhea").to_vec();
    hhea[4..6].copy_from_slice(&900i16.to_be_bytes());
    hhea[6..8].copy_from_slice(&(-300i16).to_be_bytes());
    let mut os2 = sfnt::table(&font, *b"OS/2").to_vec();
    os2[26..28].copy_from_slice(&0i16.to_be_bytes());
    let bytes = sfnt::replace_tables(
        &font,
        vec![(*b"hhea", hhea), (*b"OS/2", os2), (*b"glyf", vec![0])],
    );
    let (system, data) = registered(bytes);
    let coords = [8192];
    let value = data
        .unsynthesized_instance(1000.0, &coords)
        .unwrap()
        .upright_metrics(system.font_catalogue_identity(), script())
        .unwrap();
    geometry(&value, [-301.0, 702.0, 200.5, -501.5, 501.5]);
    assert_eq!(value.provenance(), UprightMetricProvenance::VheaTypographic);
}

#[test]
fn once_scaled_cross_metrics_remain_finite_and_nonzero_at_actual_instance_size_extremes() {
    let (system, data) = registered(fixture::native(0x0001_1000, 700, -300, 0));
    for size in [f32::from_bits(1), f32::MIN_POSITIVE, f32::MAX] {
        let value = data
            .unsynthesized_instance(size, &[])
            .unwrap()
            .upright_metrics(system.font_catalogue_identity(), script())
            .unwrap();
        let scale = f64::from(size) / 1000.0;
        for (actual, design) in [
            value.em_under(),
            value.em_over(),
            value.central_coordinate_or_distance(),
            value.used_under(),
            value.used_over(),
        ]
        .into_iter()
        .zip([-300.0, 700.0, 200.0, -500.0, 500.0])
        {
            let expected = design * scale;
            assert!(actual.is_finite() && actual.abs() > 0.0);
            assert!((actual - expected).abs() <= expected.abs() * 1e-12);
        }
        assert_eq!(value.instance().font_size(), size);
    }
}

#[test]
fn absence_and_version_one_interline_fields_use_centered_odd_em_without_ad_or_outline() {
    for native in [false, true] {
        let base = if native {
            fixture::native(0x0001_0000, 1200, -300, 900)
        } else {
            vertical::absent()
        };
        let bytes = fixture::odd_em(&base);
        for dirty in [false, true] {
            let bytes = if dirty {
                let mut hhea = sfnt::table(&bytes, *b"hhea").to_vec();
                hhea[4..6].copy_from_slice(&900i16.to_be_bytes());
                hhea[6..8].copy_from_slice(&(-300i16).to_be_bytes());
                sfnt::replace_tables(
                    &bytes,
                    vec![(*b"hhea", hhea), (*b"glyf", vec![0]), (*b"MVAR", vec![0])],
                )
            } else {
                bytes.clone()
            };
            let (system, data) = registered(bytes);
            for (size, half) in [(1001.0, 500.5), (10.0, 5.0)] {
                let value = data
                    .unsynthesized_instance(size, &[])
                    .unwrap()
                    .upright_metrics(system.font_catalogue_identity(), script())
                    .unwrap();
                geometry(&value, [-half, half, 0.0, -half, half]);
                assert_eq!(
                    value.reference(),
                    UprightMetricReference::NominalVerticalPenX
                );
                assert_eq!(
                    value.provenance(),
                    UprightMetricProvenance::SynthesizedUprightEmAtNominalPen
                );
                assert_eq!(value.units_per_em(), 1001);
            }
        }
    }
}

#[test]
fn vhea_mvar_uses_whole_unit_half_rounding_and_never_consumes_gap_or_horizontal_effects() {
    let bytes = fixture::variable(&[(*b"vasc", 3), (*b"vdsc", -3), (*b"vlgp", 1000)]);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.fvar().unwrap().axis_count(), 1);
    assert_eq!(font.mvar().unwrap().value_record_count(), 3);
    let (system, data) = registered(bytes);
    let stats = system.stats();
    for (coords, expected) in [
        (0, [-300.0, 700.0, 200.0, -500.0, 500.0]),
        (8192, [-301.0, 702.0, 200.5, -501.5, 501.5]),
        (16384, [-303.0, 703.0, 200.0, -503.0, 503.0]),
    ] {
        let coordinates = [coords];
        let value = data
            .unsynthesized_instance(1000.0, &coordinates)
            .unwrap()
            .upright_metrics(system.font_catalogue_identity(), script())
            .unwrap();
        geometry(&value, expected);
        assert_eq!(value.provenance(), UprightMetricProvenance::VheaTypographic);
        assert_eq!(
            value.instance().normalized_coords().collect::<Vec<_>>(),
            coordinates
        );
    }
    assert_eq!(system.stats(), stats);
    for records in [&[][..], &[(*b"vlgp", 1000)][..]] {
        let (system, data) = registered(fixture::variable(records));
        let coordinates = [16384];
        let value = data
            .unsynthesized_instance(1000.0, &coordinates)
            .unwrap()
            .upright_metrics(system.font_catalogue_identity(), script())
            .unwrap();
        geometry(&value, [-300.0, 700.0, 200.0, -500.0, 500.0]);
    }
}

#[test]
fn consumed_base_vhea_and_mvar_failures_keep_exact_owner_without_synthesis() {
    let good = fixture::native(0x0001_1000, 700, -300, 0);
    let mut unsupported = sfnt::table(&good, *b"vhea").to_vec();
    unsupported[..4].copy_from_slice(&0x0002_0000u32.to_be_bytes());
    for (bytes, table, reason) in [
        (
            sfnt::replace_tables(&good, vec![(*b"BASE", vec![0])]),
            FontMetricTable::Base,
            FontMetricTableError::Malformed,
        ),
        (
            sfnt::replace_tables(&good, vec![(*b"vhea", vec![0])]),
            FontMetricTable::Vhea,
            FontMetricTableError::Malformed,
        ),
        (
            sfnt::replace_tables(&good, vec![(*b"vhea", unsupported)]),
            FontMetricTable::Vhea,
            FontMetricTableError::UnsupportedVersion,
        ),
        (
            sfnt::replace_tables(&good, vec![(*b"MVAR", fixture::mvar(&[(*b"vasc", 3)]))]),
            FontMetricTable::Mvar,
            FontMetricTableError::VariationWithoutAxes,
        ),
    ] {
        let (system, data) = registered(bytes);
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        table_error(
            instance
                .upright_metrics(system.font_catalogue_identity(), script())
                .unwrap_err(),
            table,
            reason,
        );
    }
    let valid = fixture::mvar(&[(*b"vasc", 3), (*b"vdsc", -3)]);
    let mut unsupported = valid.clone();
    unsupported[6..8].copy_from_slice(&9u16.to_be_bytes());
    let mut bad_row = valid.clone();
    bad_row[18..20].copy_from_slice(&99u16.to_be_bytes());
    let mut bad_store = valid.clone();
    bad_store[28..30].copy_from_slice(&2u16.to_be_bytes());
    let mut bad_axes = valid.clone();
    bad_axes[40..42].copy_from_slice(&2u16.to_be_bytes());
    let mut bad_version = valid.clone();
    bad_version[..2].copy_from_slice(&2u16.to_be_bytes());
    for (mvar, expected) in [
        (vec![0], FontMetricTableError::Malformed),
        (
            unsupported,
            FontMetricTableError::UnsupportedRecordSize { size: 9 },
        ),
        (
            bad_row,
            FontMetricTableError::VariationDeltaIndex {
                record: 0,
                outer: 0,
                inner: 99,
            },
        ),
        (
            bad_store,
            FontMetricTableError::VariationStoreFormat { format: 2 },
        ),
        (
            bad_axes,
            FontMetricTableError::VariationAxisCount {
                expected: 1,
                actual: 2,
            },
        ),
        (bad_version, FontMetricTableError::UnsupportedVersion),
    ] {
        let bytes = sfnt::replace_tables(&fixture::variable(&[]), vec![(*b"MVAR", mvar)]);
        let (system, data) = registered(bytes);
        let coordinates = [0];
        let value = data.unsynthesized_instance(10.0, &coordinates).unwrap();
        table_error(
            value
                .upright_metrics(system.font_catalogue_identity(), script())
                .unwrap_err(),
            FontMetricTable::Mvar,
            expected,
        );
    }
}

#[test]
fn glyph_free_vhea_fields_do_not_consume_missing_or_malformed_per_glyph_vmtx() {
    let good = fixture::native(0x0001_1000, 700, -300, 0);
    for (bytes, table, reason) in [
        (
            sfnt::remove_tables(&good, &[*b"vmtx"]),
            FontMetricTable::Vhea,
            FontMetricTableError::MissingCompanion {
                required: FontMetricTable::Vmtx,
            },
        ),
        (
            sfnt::replace_tables(&good, vec![(*b"vmtx", vec![0])]),
            FontMetricTable::Vmtx,
            FontMetricTableError::Malformed,
        ),
    ] {
        let (system, data) = registered(bytes);
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        let observed = instance
            .upright_metrics(system.font_catalogue_identity(), script())
            .unwrap();
        geometry(&observed, [-3.0, 7.0, 2.0, -5.0, 5.0]);
        assert_eq!(
            observed.provenance(),
            UprightMetricProvenance::VheaTypographic
        );
        // The separate glyph operation really needs its coherent metric pair.
        // Its existing precise refusal is not transplanted into a font query.
        table_error(
            instance
                .vertical_glyph_metrics(vertical::X, system.font_catalogue_identity())
                .unwrap_err(),
            table,
            reason,
        );
    }
}

#[test]
fn used_raw_edges_preserve_one_edge_horizontal_witness_and_dflt_provenance() {
    for (horizontal, vertical, expected, provenance, absence) in [
        (
            None,
            Some(base::single_axis(&[*b"ideo"], &[0])),
            [0.0, 10.0, 5.0, -5.0, 5.0],
            FontBaselineProvenance::IdeographicBottomOneEmCssMidpoint,
            Some(FontBaselineAbsence::Name),
        ),
        (
            None,
            Some(base::single_axis(&[*b"idtp"], &[1200])),
            [0.0, 12.0, 6.0, -6.0, 6.0],
            FontBaselineProvenance::VerticalTopZeroOriginCssMidpoint,
            Some(FontBaselineAbsence::Name),
        ),
        (
            Some(base::single_axis(&[*b"ideo", *b"idtp"], &[-200, 800])),
            None,
            [0.0, 10.0, 5.0, -5.0, 5.0],
            FontBaselineProvenance::HorizontalEmBoxCssMidpoint,
            Some(FontBaselineAbsence::Axis),
        ),
    ] {
        let bytes = sfnt::replace_tables(
            &vertical::absent(),
            vec![(*b"BASE", base::base(0, horizontal, vertical, None))],
        );
        let (system, data) = registered(bytes);
        let value = data
            .unsynthesized_instance(10.0, &[])
            .unwrap()
            .upright_metrics(system.font_catalogue_identity(), script())
            .unwrap();
        geometry(&value, expected);
        assert_eq!(
            value.provenance(),
            UprightMetricProvenance::RawBaseline(provenance)
        );
        assert_eq!(value.base_absence(), absence);
    }
    let mut axis = base::single_axis(&[*b"ideo", *b"idtp"], &[0, 1200]);
    axis.scripts[0].tag = *b"DFLT";
    let bytes = sfnt::replace_tables(
        &vertical::absent(),
        vec![(*b"BASE", base::base(0, None, Some(axis), None))],
    );
    let (system, data) = registered(bytes);
    let value = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .upright_metrics(system.font_catalogue_identity(), script())
        .unwrap();
    geometry(&value, [0.0, 12.0, 6.0, -6.0, 6.0]);
    assert_eq!(value.base_script(), Some(*b"DFLT"));
}

#[test]
fn consumed_format_two_reference_refusal_cannot_fall_through_to_vhea_or_synthesis() {
    let axis = base::Axis {
        tags: vec![*b"idtp"],
        scripts: vec![base::Script {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![base::Coord::Point {
                coordinate: 1200,
                glyph: 35,
                point: 5,
            }],
        }],
    };
    let bytes = sfnt::replace_tables(
        &base::point_attached_ahem(),
        vec![(*b"BASE", base::base(0, None, Some(axis), None))],
    );
    let (system, data) = registered(bytes);
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    table_error(
        instance
            .upright_metrics(system.font_catalogue_identity(), script())
            .unwrap_err(),
        FontMetricTable::Base,
        FontMetricTableError::BaselineOutline {
            glyph: 35,
            point: 5,
            reason: FontOutlineError::Unsupported {
                capability: FontOutlineCapability::CompositeGlyph,
            },
        },
    );
}

#[test]
fn used_reference_retains_catalogue_history_and_foreign_identity_errors() {
    let (mut system, data) = registered(fixture::native(0x0001_1000, 700, -300, 0));
    let old = data.catalogue_identity().clone();
    let original = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .upright_metrics(&old, script())
        .unwrap();
    system.refresh_fonts().unwrap();
    let error = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .upright_metrics(system.font_catalogue_identity(), script())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FontCatalogueMismatch);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontCatalogueMismatch {
            expected: system.font_catalogue_identity().clone(),
            actual: old.clone()
        })
    );
    geometry(&original, [-3.0, 7.0, 2.0, -5.0, 5.0]);
    let (foreign, _) = registered(vertical::absent());
    let error = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .upright_metrics(foreign.font_catalogue_identity(), script())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FontCatalogueMismatch);
    let historical = data
        .unsynthesized_instance(10.0, &[])
        .unwrap()
        .upright_metrics(&old, script())
        .unwrap();
    geometry(&historical, [-3.0, 7.0, 2.0, -5.0, 5.0]);
}
