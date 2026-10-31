//! Distinct common metric and font-design origins with a real BASE table.
use super::*;
use skrifa::raw::TableProvider;

fn font_with_horizontal_romn_300() -> Vec<u8> {
    let original = composition_font();
    let bytes = sfnt::replace_tables(
        &original,
        vec![(
            *b"BASE",
            base::base(
                0,
                Some(base::single_axis(&[*b"romn"], &[300])),
                Some(base::single_axis(&[*b"ideo", *b"idtp"], &[0, 1200])),
                None,
            ),
        )],
    );
    // Add the real horizontal reference without changing actual glyphs, width
    // actions, selected family, raw horizontal metrics, or native vertical data.
    for tag in [
        *b"name", *b"cmap", *b"GSUB", *b"hhea", *b"hmtx", *b"OS/2", *b"maxp", *b"glyf", *b"loca",
        *b"vhea", *b"vmtx",
    ] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&original, tag));
    }
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(sfnt::u16_at(sfnt::table(&bytes, *b"OS/2"), 62) & 0x80, 0);
    assert!(parsed.table_data(skrifa::Tag::new(b"fvar")).is_none());
    let table = parsed.base().unwrap();
    let axis = table.horiz_axis().unwrap().unwrap();
    assert_eq!(
        axis.base_tag_list()
            .unwrap()
            .unwrap()
            .baseline_tags()
            .iter()
            .map(|tag| tag.get().to_be_bytes())
            .collect::<Vec<_>>(),
        [*b"romn"],
    );
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(scripts.base_script_records().len(), 1);
    let script = &scripts.base_script_records()[0];
    assert_eq!(script.base_script_tag(), skrifa::Tag::new(b"latn"));
    let values = script
        .base_script(scripts.offset_data())
        .unwrap()
        .base_values()
        .unwrap()
        .unwrap();
    assert_eq!(values.default_baseline_index(), 0);
    assert_eq!(
        values
            .base_coords()
            .iter()
            .map(|coordinate| coordinate.unwrap().coordinate())
            .collect::<Vec<_>>(),
        [300],
    );
    // Existing real Shaper qualification proves normal AA35/35, raw10/10,
    // complete authored-off width receipts, and the unchanged Vertical BASE.
    provider_premises(&bytes);
    bytes
}

#[test]
fn nonzero_horizontal_base_keeps_cell_metric_reference_distinct_from_paint_origin() {
    let bytes = font_with_horizontal_romn_300();
    ordinary_control(&bytes); // Default None remains raw-zero, despite BASE300.
    let (mut system, processed, analyzed, actual_bytes) =
        source_with_font_features_and_font("AA", &OFF, bytes);
    let catalogue = system.font_catalogue_identity().clone();
    let face = system
        .registered_font_faces()
        .find(|face| face.bytes() == actual_bytes.as_slice())
        .expect("the actual injected body resource was registered")
        .to_owned();
    assert_eq!(face.index(), 0);
    let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
    assert_eq!(instance.axis_count(), 0);
    let raw = instance.metrics(&catalogue).unwrap();
    assert_eq!(
        (raw.ascent(), raw.descent(), raw.leading()),
        (8.0, -2.0, 0.0)
    );
    let baseline = instance
        .baseline(
            &catalogue,
            FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            FontBaselineAxis::HorizontalY,
            FontBaseline::Alphabetic,
        )
        .unwrap();
    assert_eq!(baseline.axis(), FontBaselineAxis::HorizontalY);
    assert_eq!(baseline.name(), FontBaseline::Alphabetic);
    let FontBaselineOutcome::Available(value) = baseline.outcome() else {
        panic!("the actual horizontal romn300 must be consumed");
    };
    assert_eq!(value.coordinate(), 3.0); // 300 * size10 / UPEM1000.
    assert_eq!(value.provenance(), FontBaselineProvenance::BaseCoordinate);
    assert_eq!(value.base_script(), Some(*b"latn"));
    assert_eq!(value.absence(), None);

    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let container = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(container.font_size(), 10.0);
    assert_eq!(
        container.effective_instance().unwrap().face().bytes(),
        actual_bytes.as_slice()
    );
    let origin_views = prepared.origin_views().collect::<Vec<_>>();
    let [view] = origin_views.as_slice() else {
        panic!("one genuine full-AA original")
    };
    assert_eq!(view.source(), processed.source());
    assert_eq!(view.local_range(), Range::new(0, 2));
    assert_eq!(view.aggregate_range(), Range::new(0, 2));
    assert_eq!(
        view.capture_position(),
        Some(processed.captured_origin(0).unwrap())
    );
    let origin = view.handle();
    let cells = prepared.cell_views().collect::<Vec<_>>();
    assert_eq!(cells.len(), 1);
    assert_eq!(cells[0].extent(), 10.0);
    assert!(cells[0].has_provider_input());

    let line = complete_line(&prepared);
    one_em(&line);
    let runs = line.run_views().collect::<Vec<_>>();
    let [run] = runs.as_slice() else {
        panic!("one retained normal AA provider run")
    };
    assert!(run.cell().is_some());
    assert_eq!(run.face().bytes(), actual_bytes.as_slice());
    assert_eq!(run.font_size(), 10.0);
    assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(run.origin(), Point::new(0.0, 3.0));
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.advance(),
                glyph.x(),
                glyph.y()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 10.0, 0.0, 0.0), (35, 10.0, 10.0, 0.0, 0.0)],
    );
    // Inspect the real retained provider, not a fabricated metric row. Native
    // baseline adjustment gives A'=8-3=5, D'=2+3=5. Centering that band in E10
    // puts the common reference at5, while paint design zero remains5+3=8.
    let actual_run = &line.inner.data.runs[0];
    let retained = line.inner.data.font_runs[actual_run.run].metrics;
    assert!(retained.horizontal().is_some());
    assert_eq!(retained.font_origin_offset(), 3.0);
    assert_eq!((retained.ascent, retained.descent), (5.0, 5.0));
    let cell = &line.inner.data.cells[0];
    let frame = cell.frame.expect("real provider frame");
    assert_eq!(
        (frame.x_scale, frame.x_offset, frame.baseline),
        (0.5, 0.0, 5.0)
    );
    assert_eq!(cell.scope_displacement.get(), 0.0);
    drop(runs);
    drop(system);

    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    for retained_line in [line, replay] {
        // Actual Vertical BASE/Inline font and anchor retain square95,20,E10.
        // The existing placement helper independently requires paint origin28.
        let layout = placed_cell(&prepared, retained_line, &actual_bytes, 0.5);
        let packets = layout.glyph_runs();
        let [packet] = packets.as_slice() else {
            panic!("one actual horizontal paint packet")
        };
        assert_eq!(packet.glyph_frame().origin(), Point::new(95.0, 28.0));
        let result = layout
            .source_range_fragments(&origin, Range::new(0, 2))
            .unwrap();
        let units = result.units().collect::<Vec<_>>();
        assert_eq!(units.len(), 2);
        for (unit, (source, edges, bounds)) in units.iter().zip([
            (
                Range::new(0, 1),
                (0.0, 10.0),
                Rect::new(95.0, 20.0, 5.0, 10.0),
            ),
            (
                Range::new(1, 2),
                (10.0, 20.0),
                Rect::new(100.0, 20.0, 5.0, 10.0),
            ),
        ]) {
            let pieces = unit.pieces().collect::<Vec<_>>();
            let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
                panic!("a real font-backed EGC must expose its own metric domain");
            };
            assert_eq!(
                piece.effective_instance().face().bytes(),
                actual_bytes.as_slice()
            );
            assert_eq!(piece.inner_advance_edges(), edges);
            assert_eq!((piece.font_ascent(), piece.font_descent()), (5.0, 5.0));
            let metric = piece.metric_frame();
            assert_eq!(metric.origin(), Point::new(95.0, 25.0));
            assert_eq!(metric.x_scale(), 0.5);
            assert_eq!(packet.glyph_frame().origin().y - metric.origin().y, 3.0);
            assert_eq!(piece.rect(), bounds);
            for intersections in [
                unit.selected_intersections().collect::<Vec<_>>(),
                unit.expanded_intersections().collect::<Vec<_>>(),
            ] {
                assert_eq!(intersections.len(), 1);
                assert_eq!(intersections[0].origin(), origin);
                assert_eq!(intersections[0].local_range(), source);
                assert_eq!(intersections[0].aggregate_range(), source);
            }
        }
    }
}
