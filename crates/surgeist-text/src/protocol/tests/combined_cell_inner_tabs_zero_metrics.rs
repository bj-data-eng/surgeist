//! Numeric TAB zero skips unavailable SPACE/ch metrics, while the actual Cell
//! still consumes its genuine captured container font and native metric receipt.
use super::*;

fn no_spacing_glyph_font() -> Vec<u8> {
    let horizontal = crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &[(65, 35)]);
    let bytes = native_metric_font(&horizontal);
    // The existing constructor changes real cmap/GSUB data; native wrapping
    // supplies real vertical tables/BASE without donating a missing SPACE.
    glyph_frame_font_premises(&bytes);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map('A'), Some(GlyphId::new(35)));
    assert!(face.charmap().map(' ').is_none());
    assert!(face.charmap().map('0').is_none());
    assert_eq!(sfnt::u16_at(sfnt::table(&bytes, *b"hmtx"), 35 * 4), 1000);
    assert_eq!(face.os2().unwrap().fs_selection().bits() & 0x80, 0);
    assert!(face.fvar().is_err());
    assert!(face.mvar().is_err());
    let font = ProviderFont::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    let mut input = UnicodeBuffer::new();
    input.add('A', 0);
    input.set_direction(ProviderDirection::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let output = shaper.shape(input, &[]);
    assert_eq!(output.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        output
            .glyph_infos()
            .iter()
            .map(|g| (g.glyph_id, g.cluster))
            .collect::<Vec<_>>(),
        [(35, 0)]
    );
    assert_eq!(
        output
            .glyph_positions()
            .iter()
            .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
            .collect::<Vec<_>>(),
        [(1000, 0, 0, 0)]
    );
    bytes
}

#[test]
fn zero_numeric_inner_tab_skips_missing_space_metrics_but_keeps_real_container_metrics() {
    let bytes = no_spacing_glyph_font();
    for orientation in [
        LineOrientation::Vertical,
        LineOrientation::SidewaysClockwise,
        LineOrientation::SidewaysCounterclockwise,
    ] {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(bytes.clone()).unwrap();
        assert_eq!(system.registered_font_faces().count(), 1);

        // The same actual font cannot provide a demanded numeric SPACE metric.
        // This is a metric capability refusal, not missing capture or Cell guard.
        let positive = captured(
            "\tA",
            TabSize::spaces(2.0).unwrap(),
            20.0,
            ParagraphBaseDirection::LeftToRight,
            TextCombineUpright::None,
            orientation,
            "Ahem",
        );
        assert_eq!(positive.analyzed.combined_cells().len(), 0);
        let error = system
            .prepare(&positive.analyzed, options(orientation))
            .unwrap_err();
        assert!(
            matches!(
                error.detail(),
                Some(ErrorDetail::InvalidTextProtocol {
                    reason: ProtocolError::UnsupportedContext {
                        field: ProtocolContextField::TabMetrics
                    }
                })
            ),
            "unexpected positive numeric reference refusal: {error:?}"
        );

        // With numeric zero no SPACE or ZERO is demanded. A remains a real
        // provider input, supplies ordinary line metrics, and starts at raw0.
        let ordinary = captured(
            "\tA",
            TabSize::spaces(0.0).unwrap(),
            20.0,
            ParagraphBaseDirection::LeftToRight,
            TextCombineUpright::None,
            orientation,
            "Ahem",
        );
        assert_eq!(ordinary.analyzed.combined_cells().len(), 0);
        let ordinary_prepared = system
            .prepare(&ordinary.analyzed, options(orientation))
            .unwrap();
        let ordinary_line = complete(&ordinary_prepared, TabContext::CssStops { origin: None });
        assert_eq!(ordinary_line.feedback().advance, 10.0);
        assert_eq!(raw_glyphs(&ordinary_line, &bytes, false), [(35, 0.0)]);
        for run in ordinary_line
            .run_views()
            .filter(|run| run.glyphs().next().is_some())
        {
            let instance = run.effective_instance().unwrap();
            let metrics = instance
                .metrics(instance.face().catalogue_identity())
                .unwrap();
            assert_eq!(metrics.line_metric_source(), FontLineMetricSource::Hhea);
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
            let baseline = instance
                .baseline(
                    instance.face().catalogue_identity(),
                    reference(orientation).script,
                    FontBaselineAxis::HorizontalY,
                    FontBaseline::Alphabetic,
                )
                .unwrap();
            let FontBaselineOutcome::Available(value) = baseline.outcome() else {
                panic!("actual Alphabetic zero");
            };
            assert_eq!(value.coordinate(), 0.0);
            assert_eq!(
                value.provenance(),
                FontBaselineProvenance::HorizontalAlphabeticZero
            );
        }

        let capture = captured(
            "\tA",
            TabSize::spaces(0.0).unwrap(),
            20.0,
            ParagraphBaseDirection::LeftToRight,
            TextCombineUpright::All,
            orientation,
            "Ahem",
        );
        source_premises(
            &capture,
            "\tA",
            ParagraphBaseDirection::LeftToRight,
            &[0, 0],
        );
        let prepared = system
            .prepare(&capture.analyzed, options(orientation))
            .expect("numeric-zero Cell TAB must not probe missing SPACE/ch metrics");
        assert_eq!(
            prepared.analyzed_source().identity(),
            capture.analyzed.identity()
        );
        assert_eq!(prepared.cell_views().len(), 1);
        let cell = prepared.cell_views().next().unwrap();
        assert!(cell.has_provider_input()); // Actual A, not a TAB provider glyph.
        assert_eq!(cell.extent(), 10.0);
        container_premises(&prepared, &capture, &bytes);
        let line = complete(&prepared, TabContext::NoTabs);
        observe(
            &prepared,
            line.clone(),
            &capture,
            &bytes,
            &[('A', 0.0)],
            Some(10.0),
        );
        drop(system);
        observe(
            &prepared,
            line.with_spacing(SpacingAllocations::None).unwrap(),
            &capture,
            &bytes,
            &[('A', 0.0)],
            Some(10.0),
        );
        source_premises(
            &capture,
            "\tA",
            ParagraphBaseDirection::LeftToRight,
            &[0, 0],
        );
    }
}
