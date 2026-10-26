//! Independent real-font inputs and literal named-baseline observations.
use crate::baseline_test_fonts as fixture;
use crate::test_fonts;
use crate::*;
use fixture::{Axis, Coord, Script as ScriptRecord};

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const H: FontBaselineAxis = FontBaselineAxis::HorizontalY;
const V: FontBaselineAxis = FontBaselineAxis::VerticalX;
const ALPHA: FontBaseline = FontBaseline::Alphabetic;
const CENTRAL: FontBaseline = FontBaseline::Central;

fn script(tag: [u8; 4]) -> FontBaselineScript {
    FontBaselineScript::from_iso15924(tag).unwrap()
}
fn registered(bytes: Vec<u8>) -> (System, FontData) {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let data = system.registered_font_faces().next().unwrap().to_owned();
    (system, data)
}
fn with_base(font: &[u8], base: Vec<u8>) -> Vec<u8> {
    test_fonts::replace_tables(font, vec![(*b"BASE", base)])
}
fn value(measurement: &FontBaselineMeasurement<'_>) -> FontBaselineValue {
    match measurement.outcome() {
        FontBaselineOutcome::Available(value) => *value,
        other => panic!("expected available baseline, got {other:?}"),
    }
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}
fn assert_value(
    measurement: &FontBaselineMeasurement<'_>,
    expected: f64,
    provenance: FontBaselineProvenance,
) {
    let value = value(measurement);
    close(value.coordinate(), expected);
    assert_eq!(value.provenance(), provenance);
}
fn assert_unavailable(
    measurement: &FontBaselineMeasurement<'_>,
    reason: FontBaselineUnavailableReason,
    absence: FontBaselineAbsence,
) {
    match measurement.outcome() {
        FontBaselineOutcome::Unavailable(value) => {
            assert_eq!(value.reason(), reason);
            assert_eq!(value.absence(), absence);
        }
        other => panic!("expected unavailable baseline, got {other:?}"),
    }
}
fn assert_table(error: Error, reason: FontMetricTableError) {
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontMetricTable {
            table: FontMetricTable::Base,
            reason
        })
    );
}

fn assert_delta_index(error: Error, outer: u16, inner: u16) {
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    match error.detail() {
        Some(ErrorDetail::FontMetricTable {
            table: FontMetricTable::Base,
            reason:
                FontMetricTableError::VariationDeltaIndex {
                    outer: actual_outer,
                    inner: actual_inner,
                    ..
                },
        }) => {
            assert_eq!((*actual_outer, *actual_inner), (outer, inner));
        }
        other => panic!("expected used BASE delta-index refusal, got {other:?}"),
    }
}
fn point_axis(coordinate: i16, glyph: u16, point: u16) -> Axis {
    Axis {
        tags: vec![*b"romn"],
        scripts: vec![ScriptRecord {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![Coord::Point {
                coordinate,
                glyph,
                point,
            }],
        }],
    }
}
fn adjusted_axis(adjustment: Option<Vec<u8>>) -> Axis {
    Axis {
        tags: vec![*b"romn"],
        scripts: vec![ScriptRecord {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![Coord::Adjusted {
                coordinate: 123,
                adjustment,
            }],
        }],
    }
}

#[test]
fn absent_base_uses_horizontal_instance_metrics_without_inventing_vertical_references() {
    let (system, data) = registered(test_fonts::remove_tables(AHEM, &[*b"BASE"]));
    let stats = system.stats();
    for (size, central) in [(10.0, 3.0), (20.0, 6.0)] {
        let instance = data.unsynthesized_instance(size, &[]).unwrap();
        let alpha = instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap();
        let middle = instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                H,
                CENTRAL,
            )
            .unwrap();
        assert_value(
            &alpha,
            0.0,
            FontBaselineProvenance::HorizontalAlphabeticZero,
        );
        assert_value(
            &middle,
            central,
            FontBaselineProvenance::InstanceMetricsCssMidpoint,
        );
        assert_eq!(
            value(&middle).absence(),
            Some(FontBaselineAbsence::BaseTable)
        );
        assert_eq!(value(&middle).base_script(), None);
        assert_eq!(middle.instance().font_size(), size);
        assert!(middle.instance().face().is_same_face(&data.face()));
        assert_eq!(middle.script().iso15924(), *b"Latn");
        assert_eq!((middle.axis(), middle.name()), (H, CENTRAL));
        assert_unavailable(
            &instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), V, ALPHA)
                .unwrap(),
            FontBaselineUnavailableReason::VerticalAlphabetic,
            FontBaselineAbsence::BaseTable,
        );
        assert_unavailable(
            &instance
                .baseline(
                    system.font_catalogue_identity(),
                    script(*b"Hani"),
                    V,
                    CENTRAL,
                )
                .unwrap(),
            FontBaselineUnavailableReason::VerticalEmBoxReference,
            FontBaselineAbsence::BaseTable,
        );
    }
    assert_eq!(system.stats(), stats);
}

#[test]
fn horizontal_synthesis_uses_the_real_metric_selector_without_leading_or_em_normalization() {
    for use_typo in [false, true] {
        for (hhea_gap, typo_gap) in [(123, 900), (-100, -200)] {
            let (system, data) =
                registered(fixture::metric_discriminator(use_typo, hhea_gap, typo_gap));
            let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
            let before = instance.metrics(system.font_catalogue_identity()).unwrap();
            let snapshot = (
                before.ascent(),
                before.descent(),
                before.leading(),
                before.strikeout().center_offset(),
            );
            close(before.ascent(), if use_typo { 7.0 } else { 9.0 });
            close(before.descent(), if use_typo { -2.0 } else { -3.0 });
            let measured = instance
                .baseline(
                    system.font_catalogue_identity(),
                    script(*b"Latn"),
                    H,
                    CENTRAL,
                )
                .unwrap();
            assert_value(
                &measured,
                if use_typo { 2.5 } else { 3.0 },
                FontBaselineProvenance::InstanceMetricsCssMidpoint,
            );
            let after = instance.metrics(system.font_catalogue_identity()).unwrap();
            assert_eq!(
                (
                    after.ascent(),
                    after.descent(),
                    after.leading(),
                    after.strikeout().center_offset()
                ),
                snapshot
            );
        }
    }
}

#[test]
fn horizontal_fallback_consumes_the_actual_instance_mvar_metrics() {
    let (system, data) = registered(fixture::variable_metric_ahem());
    let stats = system.stats();
    for (coordinates, ascent, expected) in [
        (&[0][..], 8.0, 3.0),
        (&[8192][..], 8.5, 3.25),
        (&[16384][..], 9.0, 3.5),
    ] {
        let instance = data.unsynthesized_instance(10.0, coordinates).unwrap();
        let metrics = instance.metrics(system.font_catalogue_identity()).unwrap();
        assert_eq!(metrics.ascent(), ascent);
        assert_eq!(metrics.descent(), -2.0);
        assert_value(
            &instance
                .baseline(
                    system.font_catalogue_identity(),
                    script(*b"Latn"),
                    H,
                    CENTRAL,
                )
                .unwrap(),
            expected,
            FontBaselineProvenance::InstanceMetricsCssMidpoint,
        );
    }
    assert_eq!(system.stats(), stats);
}

#[test]
fn distinct_axis_edges_and_nonzero_romn_keep_their_original_font_design_origins() {
    let (system, data) = registered(with_base(AHEM, fixture::full_base()));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    let ha = instance
        .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
        .unwrap();
    let hc = instance
        .baseline(
            system.font_catalogue_identity(),
            script(*b"Latn"),
            H,
            CENTRAL,
        )
        .unwrap();
    let va = instance
        .baseline(system.font_catalogue_identity(), script(*b"Latn"), V, ALPHA)
        .unwrap();
    let vc = instance
        .baseline(
            system.font_catalogue_identity(),
            script(*b"Latn"),
            V,
            CENTRAL,
        )
        .unwrap();
    for (measurement, expected, provenance) in [
        (&ha, 0.25, FontBaselineProvenance::BaseCoordinate),
        (&hc, 3.0, FontBaselineProvenance::BaseEdgesCssMidpoint),
        (&va, 3.5, FontBaselineProvenance::BaseCoordinate),
        (&vc, 6.0, FontBaselineProvenance::BaseEdgesCssMidpoint),
    ] {
        assert_value(measurement, expected, provenance);
        assert_eq!(value(measurement).base_script(), Some(*b"latn"));
        assert_eq!(value(measurement).absence(), None);
    }
    close(value(&hc).coordinate() - value(&ha).coordinate(), 2.75);
    close(value(&vc).coordinate() - value(&va).coordinate(), 2.5);
    let metrics = instance.metrics(system.font_catalogue_identity()).unwrap();
    assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
}

#[test]
fn each_available_edge_and_horizontal_em_witness_has_its_own_synthesis_precedence() {
    for (axis, tag, coordinate, expected, provenance) in [
        (
            H,
            *b"ideo",
            -120,
            3.8,
            FontBaselineProvenance::IdeographicBottomOneEmCssMidpoint,
        ),
        (
            H,
            *b"idtp",
            900,
            4.0,
            FontBaselineProvenance::IdeographicTopOneEmCssMidpoint,
        ),
        (
            V,
            *b"idtp",
            1200,
            6.0,
            FontBaselineProvenance::VerticalTopZeroOriginCssMidpoint,
        ),
        (
            V,
            *b"ideo",
            0,
            5.0,
            FontBaselineProvenance::IdeographicBottomOneEmCssMidpoint,
        ),
    ] {
        let edge = fixture::single_axis(&[tag], &[coordinate]);
        let bytes = if axis == H {
            fixture::base(0, Some(edge), None, None)
        } else {
            fixture::base(0, None, Some(edge), None)
        };
        let (system, data) = registered(with_base(AHEM, bytes));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_value(
            &instance
                .baseline(
                    system.font_catalogue_identity(),
                    script(*b"Latn"),
                    axis,
                    CENTRAL,
                )
                .unwrap(),
            expected,
            provenance,
        );
        if axis == V {
            assert_unavailable(
                &instance
                    .baseline(system.font_catalogue_identity(), script(*b"Latn"), V, ALPHA)
                    .unwrap(),
                FontBaselineUnavailableReason::VerticalAlphabetic,
                FontBaselineAbsence::Name,
            );
        }
    }
    for (tag, coordinate, available) in [
        (*b"ideo", -120, true),
        (*b"idtp", 900, false),
        (*b"romn", 25, false),
    ] {
        let (system, data) = registered(with_base(
            AHEM,
            fixture::base(
                0,
                Some(fixture::single_axis(&[tag], &[coordinate])),
                None,
                None,
            ),
        ));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        let vertical = instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                V,
                CENTRAL,
            )
            .unwrap();
        if available {
            assert_value(
                &vertical,
                5.0,
                FontBaselineProvenance::HorizontalEmBoxCssMidpoint,
            );
        } else {
            assert_unavailable(
                &vertical,
                FontBaselineUnavailableReason::VerticalEmBoxReference,
                FontBaselineAbsence::Axis,
            );
        }
    }
    let (system, data) = registered(with_base(
        AHEM,
        fixture::base(
            0,
            None,
            Some(fixture::single_axis(&[*b"romn"], &[350])),
            None,
        ),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_value(
        &instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), V, ALPHA)
            .unwrap(),
        3.5,
        FontBaselineProvenance::BaseCoordinate,
    );
    assert_unavailable(
        &instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                V,
                CENTRAL,
            )
            .unwrap(),
        FontBaselineUnavailableReason::VerticalEmBoxReference,
        FontBaselineAbsence::Name,
    );
}

#[test]
fn signed_odd_edges_keep_exact_css_midpoints_before_scaling() {
    for (bottom, top, expected) in [
        (0, 1001, 5.005),
        (-1001, 0, -5.005),
        (32766, 32767, 327.665),
        (-32768, -32767, -327.675),
    ] {
        let base = fixture::base(
            0,
            Some(fixture::single_axis(&[*b"ideo", *b"idtp"], &[bottom, top])),
            None,
            None,
        );
        let (system, data) = registered(with_base(AHEM, base));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_value(
            &instance
                .baseline(
                    system.font_catalogue_identity(),
                    script(*b"Latn"),
                    H,
                    CENTRAL,
                )
                .unwrap(),
            expected,
            FontBaselineProvenance::BaseEdgesCssMidpoint,
        );
    }
}

#[test]
fn real_unicode_script_mapping_prefers_legacy_then_dflt_and_ignores_default_names() {
    let tags = vec![*b"cent", *b"icfb", *b"icft", *b"ideo", *b"idtp", *b"romn"];
    let records = [
        (*b"DFLT", -100, 900, 10),
        (*b"cyrl", -400, 600, 75),
        (*b"deva", 0, 1200, 90),
        (*b"dev2", 0, 1600, 95),
        (*b"kana", -300, 700, 50),
        (*b"latn", -200, 800, 25),
    ];
    // Script records are sorted by actual tag bytes, independently of input order.
    let mut scripts = records
        .into_iter()
        .map(|(tag, bottom, top, romn)| ScriptRecord {
            tag,
            default_index: 0,
            coordinates: [7777, -3000, 3000, bottom, top, romn]
                .into_iter()
                .map(Coord::Design)
                .collect(),
        })
        .collect::<Vec<_>>();
    scripts.sort_by_key(|record| record.tag);
    let (system, data) = registered(with_base(
        AHEM,
        fixture::base(0, Some(Axis { tags, scripts }), None, None),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    for (iso, selected, central, alpha) in [
        (*b"Latn", *b"latn", 3.0, 0.25),
        (*b"Cyrl", *b"cyrl", 1.0, 0.75),
        (*b"Hira", *b"kana", 2.0, 0.5),
        (*b"Kana", *b"kana", 2.0, 0.5),
        (*b"Deva", *b"deva", 6.0, 0.9),
        (*b"Grek", *b"DFLT", 4.0, 0.1),
    ] {
        let middle = instance
            .baseline(system.font_catalogue_identity(), script(iso), H, CENTRAL)
            .unwrap();
        assert_value(
            &middle,
            central,
            FontBaselineProvenance::BaseEdgesCssMidpoint,
        );
        assert_eq!(value(&middle).base_script(), Some(selected));
        assert_eq!(middle.script().iso15924(), iso);
        assert_value(
            &instance
                .baseline(system.font_catalogue_identity(), script(iso), H, ALPHA)
                .unwrap(),
            alpha,
            FontBaselineProvenance::BaseCoordinate,
        );
    }
    assert!(FontBaselineScript::from_iso15924(*b"Qaaa").is_none());
    let scripts = vec![
        ScriptRecord {
            tag: *b"cyrl",
            default_index: 0,
            coordinates: vec![Coord::Design(-400), Coord::Design(600)],
        },
        ScriptRecord {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![Coord::Design(0), Coord::Design(1400)],
        },
    ];
    let (system, data) = registered(with_base(
        AHEM,
        fixture::base(
            0,
            Some(Axis {
                tags: vec![*b"ideo", *b"idtp"],
                scripts,
            }),
            None,
            None,
        ),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    let middle = instance
        .baseline(
            system.font_catalogue_identity(),
            script(*b"Grek"),
            H,
            CENTRAL,
        )
        .unwrap();
    assert_value(
        &middle,
        3.0,
        FontBaselineProvenance::InstanceMetricsCssMidpoint,
    );
    assert_eq!(value(&middle).absence(), Some(FontBaselineAbsence::Script));
}

#[test]
fn real_base_variation_preserves_fractional_deltas_and_complete_instance_coordinates() {
    let font = test_fonts::variable_ahem();
    let (system, data) = registered(with_base(&font, fixture::variation_base()));
    for (coordinates, canonical, expected) in [
        (&[][..], 0, 5.0),
        (&[0][..], 0, 5.0),
        (&[8192][..], 8192, 5.0075),
        (&[16384][..], 16384, 5.015),
    ] {
        let instance = data.unsynthesized_instance(10.0, coordinates).unwrap();
        let middle = instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                H,
                CENTRAL,
            )
            .unwrap();
        assert_value(
            &middle,
            expected,
            FontBaselineProvenance::BaseEdgesCssMidpoint,
        );
        assert_eq!(
            middle.instance().normalized_coords().collect::<Vec<_>>(),
            [canonical]
        );
    }
    let font = fixture::two_axis_ahem();
    for (first, coordinates, expected) in [
        ([0, 16384, 16384], [8192, 8192], 5.00375),
        ([8192, 4096, 16384], [0, 8192], 5.0075),
        ([-16384, 8192, 16384], [0, 8192], 5.0075),
        ([0, 0, 16384], [0, 8192], 5.0075),
    ] {
        let h = Axis {
            tags: vec![*b"ideo", *b"idtp"],
            scripts: vec![ScriptRecord {
                tag: *b"latn",
                default_index: 0,
                coordinates: vec![
                    Coord::Design(0),
                    Coord::Adjusted {
                        coordinate: 1000,
                        adjustment: Some(fixture::variation_index(0, 0)),
                    },
                ],
            }],
        };
        let base = fixture::base(
            1,
            Some(h),
            None,
            Some(fixture::region_store(&[first, [0, 16384, 16384]], 3)),
        );
        let (system, data) = registered(with_base(&font, base));
        let instance = data.unsynthesized_instance(10.0, &coordinates).unwrap();
        let middle = instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                H,
                CENTRAL,
            )
            .unwrap();
        assert_value(
            &middle,
            expected,
            FontBaselineProvenance::BaseEdgesCssMidpoint,
        );
        assert_eq!(
            middle.instance().normalized_coords().collect::<Vec<_>>(),
            coordinates
        );
    }
}

#[test]
fn static_and_variable_point_coordinates_are_validated_but_not_replaced_by_outline_positions() {
    for variable in [false, true] {
        let font = if variable {
            test_fonts::variable_ahem()
        } else {
            AHEM.to_vec()
        };
        let base = fixture::base(
            0,
            Some(point_axis(123, 35, 1)),
            Some(point_axis(777, 58, 1)),
            None,
        );
        let (system, data) = registered(with_base(&font, base));
        for coordinates in [&[][..], if variable { &[16384][..] } else { &[][..] }] {
            let instance = data.unsynthesized_instance(10.0, coordinates).unwrap();
            assert_value(
                &instance
                    .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                    .unwrap(),
                1.23,
                FontBaselineProvenance::BaseCoordinate,
            );
            assert_value(
                &instance
                    .baseline(system.font_catalogue_identity(), script(*b"Latn"), V, ALPHA)
                    .unwrap(),
                7.77,
                FontBaselineProvenance::BaseCoordinate,
            );
        }
        let base = fixture::base(
            0,
            Some(fixture::single_axis(&[*b"romn"], &[123])),
            None,
            None,
        );
        let (system, data) = registered(with_base(&font, base));
        let instance = data
            .unsynthesized_instance(10.0, if variable { &[16384] } else { &[] })
            .unwrap();
        assert_value(
            &instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap(),
            1.23,
            FontBaselineProvenance::BaseCoordinate,
        );
    }
    for (glyph, point, reason) in [
        (
            281,
            0,
            FontMetricTableError::BaselineOutline {
                glyph: 281,
                point: 0,
                reason: FontOutlineError::GlyphRange { count: 281 },
            },
        ),
        (
            35,
            4,
            FontMetricTableError::BaselinePointRange {
                glyph: 35,
                point: 4,
                count: 4,
            },
        ),
        (
            3,
            0,
            FontMetricTableError::BaselinePointRange {
                glyph: 3,
                point: 0,
                count: 0,
            },
        ),
    ] {
        let (system, data) = registered(with_base(
            AHEM,
            fixture::base(0, Some(point_axis(123, glyph, point)), None, None),
        ));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_table(
            instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap_err(),
            reason,
        );
    }
    let compound = fixture::compound_ahem();
    let (system, data) = registered(with_base(
        &compound,
        fixture::base(0, Some(point_axis(123, 35, 1)), None, None),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    // Identity-XY compounds are a real supported capability. Their stored
    // coordinate remains the unhinted result, just as for simple outlines.
    assert_value(
        &instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap(),
        1.23,
        FontBaselineProvenance::BaseCoordinate,
    );
    let attached = fixture::point_attached_ahem();
    let (system, data) = registered(with_base(
        &attached,
        fixture::base(0, Some(point_axis(123, 35, 5)), None, None),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_table(
        instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap_err(),
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
fn static_device_pixels_are_omitted_only_after_validation_and_variable_device_is_invalid() {
    for adjustment in [
        None,
        Some(fixture::device(10, 10, 1, &[0x4000])),
        Some(fixture::device(10, 10, 2, &[0x1000])),
        Some(fixture::device(10, 10, 3, &[0x0100])),
    ] {
        let (system, data) = registered(with_base(
            AHEM,
            fixture::base(0, Some(adjusted_axis(adjustment)), None, None),
        ));
        for (size, expected) in [(10.0, 1.23), (20.0, 2.46)] {
            let instance = data.unsynthesized_instance(size, &[]).unwrap();
            assert_value(
                &instance
                    .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                    .unwrap(),
                expected,
                FontBaselineProvenance::BaseCoordinate,
            );
        }
    }
    let font = test_fonts::variable_ahem();
    let (system, data) = registered(with_base(
        &font,
        fixture::base(1, Some(adjusted_axis(None)), None, None),
    ));
    let instance = data.unsynthesized_instance(10.0, &[16384]).unwrap();
    assert_value(
        &instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap(),
        1.23,
        FontBaselineProvenance::BaseCoordinate,
    );
    let (system, data) = registered(with_base(
        &font,
        fixture::base(
            0,
            Some(adjusted_axis(Some(fixture::device(10, 10, 1, &[0x4000])))),
            None,
            None,
        ),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_table(
        instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap_err(),
        FontMetricTableError::BaselineVariableDevice,
    );
    let (system, data) = registered(with_base(
        AHEM,
        fixture::base(
            0,
            Some(adjusted_axis(Some(fixture::device(10, 10, 4, &[0])))),
            None,
            None,
        ),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_table(
        instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap_err(),
        FontMetricTableError::BaselineDeviceFormat { format: 4 },
    );
}

fn assert_malformed_base(error: Error) {
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    assert!(
        matches!(
            error.detail(),
            Some(ErrorDetail::FontMetricTable {
                table: FontMetricTable::Base,
                reason: FontMetricTableError::Malformed | FontMetricTableError::Inaccessible,
            })
        ),
        "expected malformed/inaccessible BASE, got {error:?}"
    );
}

#[test]
fn malformed_consumed_inventory_coordinates_and_device_payload_never_synthesize() {
    let mut cases = Vec::new();
    let h = Axis {
        tags: vec![*b"romn", *b"ideo"],
        scripts: vec![ScriptRecord {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![Coord::Design(25), Coord::Design(-200)],
        }],
    };
    cases.push((
        fixture::base(0, Some(h), None, None),
        FontMetricTableError::BaselineTagOrder,
    ));
    let h = Axis {
        tags: vec![*b"romn", *b"romn"],
        scripts: vec![ScriptRecord {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![Coord::Design(25), Coord::Design(26)],
        }],
    };
    cases.push((
        fixture::base(0, Some(h), None, None),
        FontMetricTableError::BaselineTagOrder,
    ));
    for tags in [[*b"latn", *b"DFLT"], [*b"latn", *b"latn"]] {
        let scripts = tags
            .into_iter()
            .map(|tag| ScriptRecord {
                tag,
                default_index: 0,
                coordinates: vec![Coord::Design(25)],
            })
            .collect();
        cases.push((
            fixture::base(
                0,
                Some(Axis {
                    tags: vec![*b"romn"],
                    scripts,
                }),
                None,
                None,
            ),
            FontMetricTableError::BaselineScriptOrder,
        ));
    }
    let h = Axis {
        tags: vec![*b"ideo", *b"idtp", *b"romn"],
        scripts: vec![ScriptRecord {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![Coord::Design(-200), Coord::Design(800)],
        }],
    };
    cases.push((
        fixture::base(0, Some(h), None, None),
        FontMetricTableError::BaselineValueCount {
            expected: 3,
            actual: 2,
        },
    ));
    let h = Axis {
        tags: vec![*b"romn"],
        scripts: vec![ScriptRecord {
            tag: *b"latn",
            default_index: 1,
            coordinates: vec![Coord::Design(25)],
        }],
    };
    cases.push((
        fixture::base(0, Some(h), None, None),
        FontMetricTableError::BaselineDefaultIndex { index: 1, count: 1 },
    ));
    let mut unknown = fixture::full_base();
    // Authored full table: H romn Coord1 begins at byte58.
    unknown[58..60].copy_from_slice(&99u16.to_be_bytes());
    cases.push((
        unknown,
        FontMetricTableError::BaselineCoordinateFormat { format: 99 },
    ));
    for (base, expected) in cases {
        let (system, data) = registered(with_base(AHEM, base));
        let stats = system.stats();
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_table(
            instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap_err(),
            expected,
        );
        assert_eq!(system.stats(), stats);
    }
    for adjustment in [
        fixture::device(10, 9, 1, &[0x4000]),
        fixture::device(10, 20, 1, &[]),
    ] {
        let (system, data) = registered(with_base(
            AHEM,
            fixture::base(0, Some(adjusted_axis(Some(adjustment))), None, None),
        ));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_malformed_base(
            instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap_err(),
        );
    }
    let (system, data) = registered(with_base(AHEM, vec![0]));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_malformed_base(
        instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap_err(),
    );
    let base = fixture::base(
        0,
        None,
        Some(fixture::single_axis(&[*b"ideo", *b"idtp"], &[1, 1200])),
        None,
    );
    let (system, data) = registered(with_base(AHEM, base));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    let error = instance
        .baseline(
            system.font_catalogue_identity(),
            script(*b"Latn"),
            V,
            CENTRAL,
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidFontMetric {
            field: FontMetricField::BaselineIdeographicOrigin,
            phase: FontMetricPhase::Unscaled,
            category: FontMetricNumericError::NonZero,
            value: 1.0,
        })
    );
}

#[test]
fn genuine_variation_indices_validate_used_store_and_rows_even_at_default_or_inactive_coordinates()
{
    let font = test_fonts::variable_ahem();
    let original = fixture::variation_base();
    let mut cases = Vec::new();
    for (offset, value, reason) in [
        (
            64,
            2u16,
            FontMetricTableError::VariationStoreFormat { format: 2 },
        ),
        (
            76,
            2,
            FontMetricTableError::VariationAxisCount {
                expected: 1,
                actual: 2,
            },
        ),
        (
            80,
            32767,
            FontMetricTableError::VariationRegionCoordinate {
                region: 0,
                axis: 0,
                value: 32767,
            },
        ),
        (
            88,
            2,
            FontMetricTableError::VariationWordCount {
                offset: 22,
                words: 2,
                regions: 1,
            },
        ),
        (
            92,
            1,
            FontMetricTableError::VariationRegionIndex {
                offset: 22,
                index: 1,
            },
        ),
        (
            58,
            1,
            FontMetricTableError::VariationDeltaIndex {
                record: 1,
                outer: 1,
                inner: 0,
            },
        ),
        (
            60,
            1,
            FontMetricTableError::VariationDeltaIndex {
                record: 1,
                outer: 0,
                inner: 1,
            },
        ),
    ] {
        let mut base = original.clone();
        base[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        cases.push((base, reason));
    }
    let mut missing = original.clone();
    missing[8..12].fill(0);
    cases.push((missing, FontMetricTableError::BaselineVariationStoreMissing));
    let mut inaccessible = original.clone();
    inaccessible[72..76].copy_from_slice(&4096u32.to_be_bytes());
    cases.push((
        inaccessible,
        FontMetricTableError::VariationOffset { offset: 4096 },
    ));
    for (base, reason) in cases {
        let retained = base.clone();
        let (system, data) = registered(with_base(&font, base));
        let stats = system.stats();
        for coordinates in [&[][..], &[0][..]] {
            let instance = data.unsynthesized_instance(10.0, coordinates).unwrap();
            let error = instance
                .baseline(
                    system.font_catalogue_identity(),
                    script(*b"Latn"),
                    H,
                    CENTRAL,
                )
                .unwrap_err();
            if let FontMetricTableError::VariationDeltaIndex { outer, inner, .. } = &reason {
                assert_delta_index(error, *outer, *inner);
            } else {
                assert_table(error, reason);
            }
        }
        assert_eq!(test_fonts::table(data.bytes(), *b"BASE"), retained);
        assert_eq!(system.stats(), stats);
    }
    // The exact95-byte BASE retains only9 bytes at Data offset22; its declared
    // header8+signed16 row2 requires10. Structural admission fails before the
    // defensive per-row iterator could report VariationRow.
    let mut truncated = original;
    truncated.pop();
    let retained = truncated.clone();
    let (system, data) = registered(with_base(&font, truncated));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_table(
        instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                H,
                CENTRAL,
            )
            .unwrap_err(),
        FontMetricTableError::Malformed,
    );
    assert_eq!(test_fonts::table(data.bytes(), *b"BASE"), retained);
}

#[test]
fn recognized_math_script_uses_the_actual_legacy_math_tag_instead_of_lowercase_iso_or_dflt() {
    // Actual constructor admission was independently executed before adding
    // this functional case; no raw native-script identifier is fabricated.
    let math = script(*b"Zmth");
    let scripts = [(*b"DFLT", 600), (*b"math", 1200), (*b"zmth", 1800)]
        .into_iter()
        .map(|(tag, top)| ScriptRecord {
            tag,
            default_index: 0,
            coordinates: vec![Coord::Design(0), Coord::Design(top)],
        })
        .collect();
    let base = fixture::base(
        0,
        Some(Axis {
            tags: vec![*b"ideo", *b"idtp"],
            scripts,
        }),
        None,
        None,
    );
    let (system, data) = registered(with_base(AHEM, base));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    let central = instance
        .baseline(system.font_catalogue_identity(), math, H, CENTRAL)
        .unwrap();
    assert_value(&central, 6.0, FontBaselineProvenance::BaseEdgesCssMidpoint);
    assert_eq!(central.script().iso15924(), *b"Zmth");
    assert_eq!(value(&central).base_script(), Some(*b"math"));
}

#[test]
fn no_delta_and_null_outer_slot_are_real_zero_adjustments_but_not_store_validation_bypasses() {
    let font = test_fonts::variable_ahem();
    for (outer, inner, null) in [
        (u16::MAX, u16::MAX, false),
        (0, 0, true),
        (0, u16::MAX, true),
    ] {
        let mut store = fixture::one_axis_store(3);
        if null {
            store[8..12].fill(0);
        }
        let h = adjusted_axis(Some(fixture::variation_index(outer, inner)));
        let base = fixture::base(1, Some(h.clone()), None, Some(store.clone()));
        let (system, data) = registered(with_base(&font, base));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_value(
            &instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap(),
            1.23,
            FontBaselineProvenance::BaseCoordinate,
        );
        store[12..14].copy_from_slice(&2u16.to_be_bytes());
        let (system, data) = registered(with_base(
            &font,
            fixture::base(1, Some(h), None, Some(store)),
        ));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_table(
            instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap_err(),
            FontMetricTableError::VariationAxisCount {
                expected: 1,
                actual: 2,
            },
        );
        let mut bad_region = fixture::one_axis_store(3);
        if null {
            bad_region[8..12].fill(0);
        }
        bad_region[16..18].copy_from_slice(&32767u16.to_be_bytes());
        let (system, data) = registered(with_base(
            &font,
            fixture::base(
                1,
                Some(adjusted_axis(Some(fixture::variation_index(outer, inner)))),
                None,
                Some(bad_region),
            ),
        ));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_table(
            instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap_err(),
            FontMetricTableError::VariationRegionCoordinate {
                region: 0,
                axis: 0,
                value: 32767,
            },
        );
    }
    for (outer, inner) in [(u16::MAX, 0), (1, u16::MAX), (0, u16::MAX)] {
        let base = fixture::base(
            1,
            Some(adjusted_axis(Some(fixture::variation_index(outer, inner)))),
            None,
            Some(fixture::one_axis_store(3)),
        );
        let (system, data) = registered(with_base(&font, base));
        let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
        assert_delta_index(
            instance
                .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
                .unwrap_err(),
            outer,
            inner,
        );
    }
    // Outer0 is null; outer1 points outside BASE but is not consumed by this
    // query. The complete offset array and region list remain real and valid.
    let mut store = Vec::new();
    store.extend(1u16.to_be_bytes());
    store.extend(16u32.to_be_bytes());
    store.extend(2u16.to_be_bytes());
    store.extend(0u32.to_be_bytes());
    store.extend(4096u32.to_be_bytes());
    store.extend_from_slice(&fixture::one_axis_store(3)[12..22]);
    let (system, data) = registered(with_base(
        &font,
        fixture::base(
            1,
            Some(adjusted_axis(Some(fixture::variation_index(0, u16::MAX)))),
            None,
            Some(store),
        ),
    ));
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_value(
        &instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap(),
        1.23,
        FontBaselineProvenance::BaseCoordinate,
    );
}

#[test]
fn actual_run_script_and_instance_queries_agree_without_mutating_glyphs_cuts_or_cache_activity() {
    const PHRASE: &[u8] =
        include_bytes!("../../../../tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf");
    for (font, family, text, iso, expected_glyph, expected) in [
        (AHEM, "Ahem", "A", *b"Latn", 35, 3.0),
        (AHEM, "Ahem", "Υ", *b"Grek", 277, 1.0),
        (PHRASE, "Surgeist Phrase Ahem", "へ", *b"Hira", 35, 2.0),
    ] {
        let scripts = [
            (*b"grek", -400, 600),
            (*b"kana", -300, 700),
            (*b"latn", -200, 800),
        ]
        .into_iter()
        .map(|(tag, bottom, top)| ScriptRecord {
            tag,
            default_index: 0,
            coordinates: vec![Coord::Design(bottom), Coord::Design(top)],
        })
        .collect();
        let base = fixture::base(
            0,
            Some(Axis {
                tags: vec![*b"ideo", *b"idtp"],
                scripts,
            }),
            None,
            None,
        );
        let (mut system, _) = registered(with_base(font, base));
        let source = Source::identified(text, Id::from_u64(148600), 7);
        let layout = system
            .layout(
                source.clone(),
                Style {
                    font: Font::new().family(family),
                    size: 10.0,
                    line_height: LineHeight::Absolute(10.0),
                    white_space: WhiteSpace::Preserve,
                    ..Style::default()
                },
                Options::default(),
            )
            .unwrap();
        let mut runs = layout.run_views();
        let run = runs.next().unwrap();
        assert!(runs.next().is_none());
        assert_eq!(
            run.glyphs().map(|g| g.id()).collect::<Vec<_>>(),
            [expected_glyph]
        );
        let glyphs = run.glyphs().collect::<Vec<_>>();
        let cuts = layout.positions().collect::<Vec<_>>();
        let cut_geometry = cuts
            .iter()
            .map(|p| layout.position_geometry(p).unwrap().rect())
            .collect::<Vec<_>>();
        let caret = layout
            .cursor(Cursor::new(0, Affinity::After))
            .unwrap()
            .rect();
        let selection = layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(1, Affinity::Before),
            ))
            .unwrap()
            .rects()
            .to_vec();
        let lines = layout
            .lines()
            .iter()
            .map(|l| (l.range(), l.bounds(), l.baseline()))
            .collect::<Vec<_>>();
        let stats = system.stats();
        let shape_stats = system.shape_cache_hits();
        let basis = system.font_catalogue_identity().clone();
        let actual_instance = run.effective_instance().unwrap();
        let direct = actual_instance
            .baseline(&basis, script(iso), H, CENTRAL)
            .unwrap();
        let from_run = run.baseline(&basis, H, CENTRAL).unwrap();
        assert_value(
            &direct,
            expected,
            FontBaselineProvenance::BaseEdgesCssMidpoint,
        );
        assert_eq!(from_run.outcome(), direct.outcome());
        assert_eq!(from_run.script(), script(iso));
        assert_eq!(from_run.instance().font_size(), 10.0);
        assert!(
            from_run
                .instance()
                .face()
                .is_same_face(&actual_instance.face())
        );
        assert_eq!(
            from_run.instance().normalized_coords().collect::<Vec<_>>(),
            actual_instance.normalized_coords().collect::<Vec<_>>()
        );
        assert_eq!(run.glyphs().collect::<Vec<_>>(), glyphs);
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|l| (l.range(), l.bounds(), l.baseline()))
                .collect::<Vec<_>>(),
            lines
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::After))
                .unwrap()
                .rect(),
            caret
        );
        assert_eq!(
            layout
                .selection(Selection::new(
                    Cursor::new(0, Affinity::After),
                    Cursor::new(1, Affinity::Before)
                ))
                .unwrap()
                .rects(),
            selection
        );
        assert_eq!(layout.source(), &source);
        for (position, expected_rect) in cuts.iter().zip(cut_geometry) {
            assert_eq!(
                layout.position_geometry(position).unwrap().rect(),
                expected_rect
            );
        }
        assert_eq!(system.stats(), stats);
        assert_eq!(system.shape_cache_hits(), shape_stats);
        system.refresh_fonts().unwrap();
        assert_eq!(
            run.baseline(system.font_catalogue_identity(), H, CENTRAL)
                .unwrap_err()
                .code,
            ErrorCode::FontCatalogueMismatch
        );
        assert_value(
            &run.baseline(&basis, H, CENTRAL).unwrap(),
            expected,
            FontBaselineProvenance::BaseEdgesCssMidpoint,
        );
        drop(system);
        assert_value(
            &run.baseline(&basis, H, CENTRAL).unwrap(),
            expected,
            FontBaselineProvenance::BaseEdgesCssMidpoint,
        );
    }
}

#[test]
fn historical_actual_resources_keep_results_and_foreign_publications_cannot_authorize_them() {
    let (mut system, data) = registered(with_base(AHEM, fixture::full_base()));
    let basis = system.font_catalogue_identity().clone();
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    let retained = instance
        .baseline(&basis, script(*b"Latn"), H, CENTRAL)
        .unwrap();
    system.refresh_fonts().unwrap();
    let fresh = system.registered_font_faces().next().unwrap().to_owned();
    assert!(data.face().is_same_face(&fresh.face()));
    assert_ne!(data.catalogue_identity(), fresh.catalogue_identity());
    assert_eq!(
        instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                H,
                CENTRAL
            )
            .unwrap_err()
            .code,
        ErrorCode::FontCatalogueMismatch
    );
    let fresh_instance = fresh.unsynthesized_instance(10.0, &[]).unwrap();
    assert_value(
        &fresh_instance
            .baseline(
                system.font_catalogue_identity(),
                script(*b"Latn"),
                H,
                CENTRAL,
            )
            .unwrap(),
        3.0,
        FontBaselineProvenance::BaseEdgesCssMidpoint,
    );
    let (foreign, _) = registered(AHEM.to_vec());
    assert_eq!(
        foreign.font_catalogue_identity().generation(),
        basis.generation()
    );
    assert_eq!(
        instance
            .baseline(
                foreign.font_catalogue_identity(),
                script(*b"Latn"),
                H,
                CENTRAL
            )
            .unwrap_err()
            .code,
        ErrorCode::FontCatalogueMismatch
    );
    assert_value(&retained, 3.0, FontBaselineProvenance::BaseEdgesCssMidpoint);
    assert_value(
        &instance
            .baseline(&basis, script(*b"Latn"), H, CENTRAL)
            .unwrap(),
        3.0,
        FontBaselineProvenance::BaseEdgesCssMidpoint,
    );
}

#[test]
fn complete_base_coordinates_do_not_consume_unrelated_mvar_but_metric_fallback_does() {
    // Existing metrics tests admit this actual resource/instance and report
    // VariationWithoutAxes when metrics consume its malformed static MVAR.
    let base = fixture::base(
        0,
        Some(fixture::single_axis(&[*b"romn"], &[123])),
        None,
        None,
    );
    let bytes = test_fonts::replace_tables(AHEM, vec![(*b"BASE", base), (*b"MVAR", vec![0])]);
    let (system, data) = registered(bytes);
    let instance = data.unsynthesized_instance(10.0, &[]).unwrap();
    assert_value(
        &instance
            .baseline(system.font_catalogue_identity(), script(*b"Latn"), H, ALPHA)
            .unwrap(),
        1.23,
        FontBaselineProvenance::BaseCoordinate,
    );
    let error = instance
        .baseline(
            system.font_catalogue_identity(),
            script(*b"Latn"),
            H,
            CENTRAL,
        )
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontMetricTable {
            table: FontMetricTable::Mvar,
            reason: FontMetricTableError::VariationWithoutAxes
        })
    );
}
