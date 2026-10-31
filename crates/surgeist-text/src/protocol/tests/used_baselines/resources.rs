//! Actual script/resource/variation owners supply each used reference.
use super::*;

fn registered(bytes: Vec<u8>) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

fn prepare(
    system: &mut System,
    input: &AnalyzedSource,
    orientation: LineOrientation,
) -> PreparedText {
    system
        .prepare(
            input,
            PreparationOptions::default()
                .with_orientation(orientation)
                .with_baseline_reference(ALPHA),
        )
        .unwrap()
}

fn scripted(bytes: &[u8], values: &[([u8; 4], i16)]) -> Vec<u8> {
    let mut scripts = values
        .iter()
        .map(|&(tag, value)| base::Script {
            tag,
            default_index: 0,
            coordinates: vec![base::Coord::Design(777), base::Coord::Design(value)],
        })
        .collect::<Vec<_>>();
    scripts.sort_by_key(|script| script.tag);
    let bytes = sfnt::replace_tables(
        bytes,
        vec![(
            *b"BASE",
            base::base(
                0,
                Some(base::Axis {
                    tags: vec![*b"ideo", *b"romn"],
                    scripts,
                }),
                None,
                None,
            ),
        )],
    );
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    let axis = parsed.base().unwrap().horiz_axis().unwrap().unwrap();
    let scripts = axis.base_script_list().unwrap();
    let actual = scripts
        .base_script_records()
        .iter()
        .map(|record| {
            let value = record
                .base_script(scripts.offset_data())
                .unwrap()
                .base_values()
                .unwrap()
                .unwrap();
            assert_eq!(value.default_baseline_index(), 0);
            let coords = value
                .base_coords()
                .iter()
                .map(|coord| coord.unwrap().coordinate())
                .collect::<Vec<_>>();
            assert_eq!(coords.len(), 2);
            assert_eq!(coords[0], 777);
            (record.base_script_tag().to_be_bytes(), coords[1])
        })
        .collect::<Vec<_>>();
    let mut expected = values.to_vec();
    expected.sort_by_key(|entry| entry.0);
    assert_eq!(actual, expected);
    bytes
}

#[test]
fn actual_run_scripts_and_exact_dflt_keep_distinct_origins_on_one_paint_face() {
    for exact_greek in [false, true] {
        let mut records = vec![(*b"DFLT", -100), (*b"latn", 100)];
        if exact_greek {
            records.push((*b"grek", -100));
        }
        let bytes = scripted(AHEM, &records);
        let parsed = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(parsed.charmap().map('A'), Some(GlyphId::new(35)));
        assert_eq!(parsed.charmap().map('\u{3a5}'), Some(GlyphId::new(277)));
        let metrics = parsed.glyph_metrics(FontSize::unscaled(), LocationRef::default());
        for glyph in [35, 277] {
            assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(1000.0));
        }
        let mut system = registered(bytes.clone());
        let input = analyzed(Source::new("A\u{3a5}"), own_style());
        for orientation in [
            LineOrientation::Horizontal,
            LineOrientation::SidewaysClockwise,
        ] {
            let prepared = prepare(&mut system, &input, orientation);
            let line = finalize(&prepared);
            // Per-font-component maxima7/3 and9/1 =>9/3, even on one
            // face/size/brush. The BASE script changes the actual origin.
            assert_eq!(
                (
                    line.feedback().baseline,
                    line.feedback().line_height,
                    line.feedback().advance
                ),
                (9.0, 12.0, 20.0)
            );
            assert_eq!(
                line.run_views().map(|run| run.origin()).collect::<Vec<_>>(),
                [Point::new(0.0, 10.0), Point::new(10.0, 8.0)]
            );
            let layout = placed(
                &prepared,
                line,
                if orientation == LineOrientation::Horizontal {
                    70.0
                } else {
                    100.0
                },
            );
            let origins = if orientation == LineOrientation::Horizontal {
                [Point::new(20.0, 71.0), Point::new(30.0, 69.0)]
            } else {
                [Point::new(99.0, 20.0), Point::new(101.0, 30.0)]
            };
            assert_eq!(
                layout
                    .run_views()
                    .map(|run| run.origin())
                    .collect::<Vec<_>>(),
                origins
            );
            assert_eq!(
                layout
                    .glyph_runs()
                    .iter()
                    .map(|run| run.origin())
                    .collect::<Vec<_>>(),
                origins
            );
            for (i, run) in layout.run_views().enumerate() {
                assert_eq!(run.face().bytes(), bytes);
                let raw = run
                    .baseline(system.font_catalogue_identity(), AXIS, ALPHA)
                    .unwrap();
                assert_eq!(
                    raw.script().iso15924(),
                    if i == 0 { *b"Latn" } else { *b"Grek" }
                );
                let FontBaselineOutcome::Available(value) = raw.outcome() else {
                    panic!("actual script baseline missing")
                };
                assert_eq!(value.coordinate(), if i == 0 { 1.0 } else { -1.0 });
                assert_eq!(
                    value.base_script(),
                    Some(if i == 0 {
                        *b"latn"
                    } else if exact_greek {
                        *b"grek"
                    } else {
                        *b"DFLT"
                    })
                );
            }
            assert_eq!(
                layout
                    .glyph_runs()
                    .iter()
                    .flat_map(|run| run.glyphs().iter().map(|g| (g.id(), g.range())))
                    .collect::<Vec<_>>(),
                [(35, Range::new(0, 1)), (277, Range::new(1, 2))]
            );
            assert_eq!(layout.source().text().code_units(), [65, 0x3a5]);
            assert_eq!(
                layout.bounds(),
                if orientation == LineOrientation::Horizontal {
                    Rect::new(20.0, 61.0, 200.0, 12.0)
                } else {
                    Rect::new(97.0, 20.0, 12.0, 200.0)
                }
            );
        }
    }
}

#[test]
fn an_actual_fallback_face_uses_its_legacy_kana_reference_instead_of_the_primary_strut() {
    const PHRASE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf"
    ));
    let primary = scripted(AHEM, &[(*b"latn", 100)]);
    let fallback = scripted(
        PHRASE,
        &[(*b"DFLT", 600), (*b"kana", -100), (*b"latn", 300)],
    );
    let p = skrifa::FontRef::new(&primary).unwrap();
    let f = skrifa::FontRef::new(&fallback).unwrap();
    assert_eq!(p.charmap().map('\u{3078}'), None);
    assert_eq!(f.charmap().map('\u{3078}'), Some(GlyphId::new(35)));
    assert_eq!(f.head().unwrap().units_per_em(), 1000);
    assert_eq!(
        (
            f.hhea().unwrap().ascender().to_i16(),
            f.hhea().unwrap().descender().to_i16()
        ),
        (800, -200)
    );
    assert_eq!(
        f.glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(35)),
        Some(1000.0)
    );
    let mut system = registered(primary.clone());
    system.register_fonts(fallback.clone()).unwrap();
    let input = analyzed(
        Source::new("A\u{3078}"),
        Style {
            font: Font::new().family("Ahem").family("Surgeist Phrase Ahem"),
            ..own_style()
        },
    );
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        let prepared = prepare(&mut system, &input, orientation);
        let line = finalize(&prepared);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (9.0, 12.0)
        );
        let layout = placed(
            &prepared,
            line,
            if orientation == LineOrientation::Horizontal {
                70.0
            } else {
                100.0
            },
        );
        let runs = layout.run_views().collect::<Vec<_>>();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].face().bytes(), primary);
        assert_eq!(runs[1].face().bytes(), fallback);
        assert!(!runs[0].face().is_same_face(&runs[1].face()));
        let raw = runs[1]
            .baseline(system.font_catalogue_identity(), AXIS, ALPHA)
            .unwrap();
        assert_eq!(raw.script().iso15924(), *b"Hira");
        assert!(
            matches!(raw.outcome(), FontBaselineOutcome::Available(value) if value.coordinate() == -1.0 && value.base_script() == Some(*b"kana"))
        );
        assert_eq!(
            runs[1].origin(),
            if orientation == LineOrientation::Horizontal {
                Point::new(30.0, 69.0)
            } else {
                Point::new(101.0, 30.0)
            }
        );
    }
}

// The permitted operations are one exact region multiplication, coordinate
// addition/scaling and at most two final coordinate additions/subtractions.
// Four binary64 ULPs cover their rounding; no absolute epsilon or measured
// output supplies an expected value. This helper is only for positive values.
fn four_ulps(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite() && actual > 0.0 && expected > 0.0);
    assert!(
        actual.to_bits().abs_diff(expected.to_bits()) <= 4,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn actual_fractional_base_variation_is_retained_beyond_the_vertical_metric_grid() {
    let variable = sfnt::variable_ahem();
    let base = base::base(
        1,
        Some(base::Axis {
            tags: vec![*b"romn"],
            scripts: vec![base::Script {
                tag: *b"latn",
                default_index: 0,
                coordinates: vec![base::Coord::Adjusted {
                    coordinate: 100,
                    adjustment: Some(base::variation_index(0, 0)),
                }],
            }],
        }),
        None,
        Some(base::one_axis_store(3)),
    );
    let bytes = sfnt::replace_tables(&variable, vec![(*b"BASE", base)]);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(parsed.fvar().unwrap().axes().unwrap().len(), 1);
    assert_eq!(parsed.charmap().map('A'), Some(GlyphId::new(35)));
    let axis = parsed.base().unwrap().horiz_axis().unwrap().unwrap();
    let scripts = axis.base_script_list().unwrap();
    let values = scripts.base_script_records()[0]
        .base_script(scripts.offset_data())
        .unwrap()
        .base_values()
        .unwrap()
        .unwrap();
    assert_eq!(values.base_coords().get(0).unwrap().coordinate(), 100);
    // Qualify actual store data bytes independently of the resolver.
    assert_eq!(
        sfnt::table(&bytes, *b"BASE").last_chunk::<2>().unwrap(),
        &3_i16.to_be_bytes()
    );
    let mut system = registered(bytes);
    for (setting, coord, factor) in [
        ("\"wght\" 0", 0, 0.0),
        ("\"wght\" 0.5", 8192, 0.5),
        ("\"wght\" 1", 16384, 1.0),
    ] {
        let b = (100.0 + 3.0 * factor) * (10.0 / 1000.0);
        let input = analyzed(
            Source::new("A"),
            Style {
                font: Font::new().family("Ahem").variation(setting),
                ..own_style()
            },
        );
        for orientation in [
            LineOrientation::Horizontal,
            LineOrientation::SidewaysClockwise,
        ] {
            let prepared = prepare(&mut system, &input, orientation);
            let line = finalize(&prepared);
            four_ulps(line.feedback().baseline, 8.0 - b);
            four_ulps(line.feedback().line_height, 10.0);
            assert_eq!(
                line.run_views()
                    .flat_map(|run| run
                        .glyphs()
                        .map(|glyph| glyph.advance())
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                [10.0]
            );
            let layout = placed(
                &prepared,
                line,
                if orientation == LineOrientation::Horizontal {
                    70.0
                } else {
                    100.0
                },
            );
            let run = layout.run_views().next().unwrap();
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.normalized_coords().collect::<Vec<_>>(), [coord]);
            let raw = run
                .baseline(system.font_catalogue_identity(), AXIS, ALPHA)
                .unwrap();
            let FontBaselineOutcome::Available(value) = raw.outcome() else {
                panic!("actual variable romn missing")
            };
            four_ulps(value.coordinate(), b);
            assert_eq!(value.provenance(), FontBaselineProvenance::BaseCoordinate);
            four_ulps(
                if orientation == LineOrientation::Horizontal {
                    run.origin().y
                } else {
                    run.origin().x
                },
                if orientation == LineOrientation::Horizontal {
                    70.0 + b
                } else {
                    100.0 - b
                },
            );
            let metrics = instance.metrics(system.font_catalogue_identity()).unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
        }
    }
}

#[test]
fn genuine_horizontal_absence_synthesizes_zero_but_consumed_malformed_base_retains_its_error() {
    let mut system = registered(AHEM.to_vec());
    let input = analyzed(Source::new("A"), own_style());
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        let prepared = prepare(&mut system, &input, orientation);
        let line = finalize(&prepared);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (8.0, 10.0)
        );
        let layout = placed(
            &prepared,
            line,
            if orientation == LineOrientation::Horizontal {
                70.0
            } else {
                100.0
            },
        );
        let run = layout.run_views().next().unwrap();
        let raw = run
            .baseline(system.font_catalogue_identity(), AXIS, ALPHA)
            .unwrap();
        assert!(
            matches!(raw.outcome(), FontBaselineOutcome::Available(value) if value.coordinate() == 0.0 && value.provenance() == FontBaselineProvenance::HorizontalAlphabeticZero && value.absence() == Some(crate::FontBaselineAbsence::BaseTable))
        );
    }
    let bad = sfnt::replace_tables(
        AHEM,
        vec![(
            *b"BASE",
            base::base(
                0,
                Some(base::single_axis(&[*b"romn", *b"ideo"], &[100, 0])),
                None,
                None,
            ),
        )],
    );
    let mut system = registered(bad);
    let raw = system
        .prepare(&input, PreparationOptions::default())
        .unwrap();
    let retained = placed(&raw, finalize(&raw), 70.0);
    let snapshot = retained.glyph_runs();
    let error = system
        .prepare(
            &input,
            PreparationOptions::default().with_baseline_reference(ALPHA),
        )
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidFontMetrics);
    assert_eq!(
        error.detail(),
        Some(&crate::ErrorDetail::FontMetricTable {
            table: crate::FontMetricTable::Base,
            reason: crate::FontMetricTableError::BaselineTagOrder
        })
    );
    assert_eq!(retained.glyph_runs(), snapshot);
    assert_eq!(
        retained.run_views().next().unwrap().origin(),
        Point::new(20.0, 70.0)
    );
}

#[test]
fn requested_reference_pairs_preserve_raw_defaults_upright_central_and_orientation_precedence() {
    let mut system = registered(font(100));
    let horizontal = analyzed(Source::new("A"), own_style());
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        let error = system
            .prepare(
                &horizontal,
                PreparationOptions::default()
                    .with_orientation(orientation)
                    .with_baseline_reference(FontBaseline::Central),
            )
            .unwrap_err();
        assert_eq!(
            reason(&error),
            &ProtocolError::UnsupportedContext {
                field: ProtocolContextField::BaselineReference
            }
        );
    }
    let wrong_orientation = system
        .prepare(
            &horizontal,
            PreparationOptions::default()
                .with_orientation(LineOrientation::Vertical)
                .with_baseline_reference(ALPHA),
        )
        .unwrap_err();
    assert_eq!(
        reason(&wrong_orientation),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation
        }
    );
    let source = Source::identified("A", Id::from_u64(1561020), 1);
    let upright = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: own_style(),
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, 1),
                    aggregate: Range::new(0, 1),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            crate::AnalysisTypography::Upright {
                computed_direction: ParagraphBaseDirection::RightToLeft,
            },
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(
        upright.typography(),
        crate::AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::RightToLeft
        }
    );
    assert_eq!(upright.direction(), crate::Direction::LeftToRight);
    let options = PreparationOptions::default().with_orientation(LineOrientation::Vertical);
    for request in [
        options,
        options.with_baseline_reference(FontBaseline::Central),
    ] {
        let prepared = system.prepare(&upright, request).unwrap();
        assert_eq!(prepared.baseline_reference(), Some(FontBaseline::Central));
        let line = finalize(&prepared);
        assert_eq!(
            (
                line.feedback().baseline,
                line.feedback().line_height,
                line.feedback().advance
            ),
            (5.0, 10.0, 10.0)
        );
    }
    let error = system
        .prepare(&upright, options.with_baseline_reference(ALPHA))
        .unwrap_err();
    assert_eq!(
        reason(&error),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::BaselineReference
        }
    );
}
