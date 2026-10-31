//! Repeated expansion ancestry and retained semantic-zero source identity.
use super::super::word_spacing;
use super::*;
use crate::{SourceRangeCellUnitKind, SourceRangeGeometry, SourceRangeUnit};
use skrifa::{GlyphId, MetadataProvider};

fn issued_range_origin(
    prepared: &PreparedText,
    processed: &ProcessedSource,
    ordinal: usize,
) -> PreparedOrigin {
    let captured = processed.captured_origin(ordinal).unwrap();
    let view = prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&captured))
        .unwrap();
    assert_eq!(view.source(), processed.source());
    assert_eq!(view.local_range(), Range::new(ordinal, ordinal + 1));
    assert_eq!(view.aggregate_range(), Range::new(ordinal, ordinal + 1));
    view.handle()
}

fn exact_original(unit: SourceRangeUnit<'_>, origin: &PreparedOrigin, range: Range) {
    for pieces in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].origin(), *origin);
        assert_eq!(pieces[0].local_range(), range);
        assert_eq!(pieces[0].aggregate_range(), range);
    }
    assert_eq!(unit.controls().len(), 0);
    assert_eq!(unit.virtual_separators().len(), 0);
    assert_eq!(unit.alternatives().len(), 0);
    assert_eq!(unit.hyphenation_cuts().len(), 0);
}

#[test]
fn uppercase_case_expansion_keeps_two_semantic_units_per_actual_original() {
    for (text, originals) in [("ß", 1), ("ßß", 2)] {
        // This maintained helper REALLY authors Uppercase in the captured Inline.
        // It already publicly qualifies exact body S/SS/SSSS normal1000 per input
        // and Complete/NoAction on hwid/twid/qwid; only A has width substitutions.
        let (mut system, fixture) =
            fixture_with_processing(text, 0.0, false, false, 10.0, WhiteSpace::Preserve, true);
        assert_eq!(fixture.processed.source().text(), &Text::from(text));
        assert_eq!(fixture.processed.processed_text(), &Text::from(text));
        let cells = fixture.analyzed.combined_cells().collect::<Vec<_>>();
        let [source_cell] = cells.as_slice() else {
            panic!("one actual complete All Cell");
        };
        let count = originals * 2;
        assert_eq!(
            source_cell.scope(),
            fixture.processed.captured_scope(1).unwrap()
        );
        assert_eq!(source_cell.selection_unit_count(), count);
        assert_eq!(source_cell.used_unit_count(), count);
        assert_eq!(
            source_cell.used_code_points().collect::<Vec<_>>(),
            vec![0x53; count]
        );
        assert_eq!(
            source_cell.presentation_code_points().collect::<Vec<_>>(),
            vec![0x53; count]
        );
        let ancestry = source_cell.source_pieces().collect::<Vec<_>>();
        assert_eq!(ancestry.len(), count);
        for (ordinal, piece) in ancestry.iter().enumerate() {
            let input = ordinal / 2;
            assert_eq!(piece.source(), fixture.processed.source());
            assert_eq!(piece.local(), Range::new(input, input + 1));
            assert_eq!(piece.aggregate(), Range::new(input, input + 1));
            assert_eq!(
                piece.origin_position(),
                fixture.processed.captured_origin(input).unwrap()
            );
            assert_eq!(
                piece.paragraph(),
                fixture.processed.captured_paragraph(0).unwrap()
            );
            assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
            assert!(piece.is_used());
            assert!(piece.has_presentation_emission());
            assert_eq!(piece.emission_subposition(), Some(ordinal % 2));
        }
        let face = skrifa::FontRef::new(fixture.body.bytes()).unwrap();
        assert_eq!(face.charmap().map('S'), Some(GlyphId::new(53)));
        assert_eq!(
            sfnt::u16_at(sfnt::table(fixture.body.bytes(), *b"hmtx"), 53 * 4),
            1000
        );
        let prepared = system
            .prepare(
                &fixture.analyzed,
                PreparationOptions::default().with_orientation(LineOrientation::Vertical),
            )
            .unwrap();
        assert_eq!(prepared.origin_views().count(), originals);
        let origins = (0..originals)
            .map(|ordinal| issued_range_origin(&prepared, &fixture.processed, ordinal))
            .collect::<Vec<_>>();
        if originals == 2 {
            assert_ne!(origins[0], origins[1]);
            assert_ne!(
                fixture.processed.captured_origin(0),
                fixture.processed.captured_origin(1)
            );
        }
        let line = finalize(&prepared);
        one_em(&line);
        let raw = line
            .run_views()
            .flat_map(|run| {
                assert!(run.cell().is_some());
                assert!(run.face().is_same_face(&fixture.body.face()));
                assert_eq!(run.font_size(), 10.0);
                let instance = run.effective_instance().unwrap();
                let metrics = instance
                    .metrics(instance.face().catalogue_identity())
                    .unwrap();
                assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
                run.glyphs()
                    .map(|glyph| {
                        (
                            glyph.range(),
                            glyph.id(),
                            run.origin().x + glyph.pen(),
                            glyph.advance(),
                            glyph.x(),
                            glyph.y(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            raw,
            (0..count)
                .map(|ordinal| (
                    Range::new(ordinal / 2, ordinal / 2 + 1),
                    53,
                    10.0 * f64::from(u32::try_from(ordinal).unwrap()),
                    10.0,
                    0.0,
                    0.0
                ))
                .collect::<Vec<_>>()
        );
        let layout = placed_vertical_cell(&prepared, line);
        assert_eq!(layout.source(), fixture.processed.source());
        // Normal W=10*(2N), E10; complete negative trials retain that packet.
        // The actual metric band8/+2 and baseline(95,28) give width10/(2N).
        let scale = 1.0 / f64::from(u32::try_from(count).unwrap());
        let width = 10.0 / f64::from(u32::try_from(count).unwrap());
        let results = origins
            .iter()
            .enumerate()
            .map(|(input, origin)| {
                let result = layout
                    .source_range_fragments(origin, Range::new(input, input + 1))
                    .unwrap();
                assert_eq!(result.origin(), origin);
                assert_eq!(result.local_range(), Range::new(input, input + 1));
                assert_eq!(result.units().len(), 2);
                result
            })
            .collect::<Vec<_>>();
        let units = results
            .iter()
            .flat_map(|result| result.units())
            .collect::<Vec<_>>();
        assert_eq!(units.len(), count);
        for (ordinal, unit) in units.iter().copied().enumerate() {
            let input = ordinal / 2;
            exact_original(unit, &origins[input], Range::new(input, input + 1));
            assert_eq!(
                (
                    unit.presentation_range().start(),
                    unit.presentation_range().end()
                ),
                (ordinal, ordinal + 1)
            );
            let owner = unit.cell_unit().unwrap();
            assert_eq!(owner.kind(), SourceRangeCellUnitKind::Semantic { ordinal });
            assert_eq!((owner.line_index(), owner.cell_index()), (0, 0));
            assert_eq!(
                owner.source_cell().scope(),
                fixture.processed.captured_scope(1).unwrap()
            );
            assert_eq!(owner.source_cell().used_unit_count(), count);
            assert_eq!(owner.presentation_range(), unit.presentation_range());
            let pieces = unit.pieces().collect::<Vec<_>>();
            let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
                panic!("each expanded S has its genuine horizontal font metric piece");
            };
            assert!(piece.cell_unit().same_unit(&owner));
            assert_eq!(
                piece.inner_advance_edges(),
                (
                    10.0 * f64::from(u32::try_from(ordinal).unwrap()),
                    10.0 * f64::from(u32::try_from(ordinal + 1).unwrap())
                )
            );
            assert!(piece.inner_cluster_indices().len() > 0);
            assert!(
                piece
                    .effective_instance()
                    .face()
                    .is_same_face(&fixture.body.face())
            );
            assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
            assert_eq!(piece.metric_frame().origin(), Point::new(95.0, 28.0));
            assert_eq!(piece.metric_frame().x_scale(), scale);
            assert_eq!(
                piece.rect(),
                Rect::new(
                    95.0 + width * f64::from(u32::try_from(ordinal).unwrap()),
                    20.0,
                    width,
                    10.0
                )
            );
        }
        // Same source interval, capture and containing Cell do not merge SS EGCs.
        for first in 0..count {
            for second in 0..count {
                assert_eq!(units[first].same_unit(&units[second]), first == second);
                assert_eq!(
                    units[first]
                        .cell_unit()
                        .unwrap()
                        .same_unit(&units[second].cell_unit().unwrap()),
                    first == second
                );
            }
        }
        drop(system);
        for (input, origin) in origins.iter().enumerate() {
            let again = layout
                .source_range_fragments(origin, Range::new(input, input + 1))
                .unwrap();
            for (first, repeated) in results[input].units().zip(again.units()) {
                assert!(first.same_unit(&repeated));
                assert!(
                    first
                        .cell_unit()
                        .unwrap()
                        .same_unit(&repeated.cell_unit().unwrap())
                );
            }
        }
    }
}

fn zero_space_provider_premises(bytes: &[u8]) {
    use surgeist_shaper::{
        Direction, Feature, FontRef, ShaperData, ShapingStatus, Tag, UnicodeBuffer, WidthFeature,
        WidthInputOutcome, WidthObservationCoverage, script,
    };
    glyph_frame_font_premises(bytes);
    let face = skrifa::FontRef::new(bytes).unwrap();
    for (ch, glyph) in [('A', 35), (' ', 3)] {
        assert_eq!(face.charmap().map(ch), Some(GlyphId::new(glyph)));
        assert_eq!(
            sfnt::u16_at(
                sfnt::table(bytes, *b"hmtx"),
                usize::try_from(glyph).unwrap() * 4
            ),
            1000
        );
    }
    let font = FontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    for request in [
        None,
        Some((*b"hwid", WidthFeature::HalfWidth)),
        Some((*b"twid", WidthFeature::ThirdWidth)),
        Some((*b"qwid", WidthFeature::QuarterWidth)),
    ] {
        let mut input = UnicodeBuffer::new();
        for (index, ch) in "A A".chars().enumerate() {
            input.add(ch, u32::try_from(index).unwrap());
        }
        input.set_direction(Direction::LeftToRight);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        let features = if let Some((tag, feature)) = request {
            input.observe_width_feature(feature);
            vec![Feature::new(Tag::new(&tag), 1, ..)]
        } else {
            vec![]
        };
        let output = shaper.shape(input, &features);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(35, 0), (3, 1), (35, 2)]
        );
        assert_eq!(
            output
                .glyph_positions()
                .iter()
                .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
                .collect::<Vec<_>>(),
            [(1000, 0, 0, 0); 3]
        );
        if let Some((_, feature)) = request {
            let receipt = output.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.feature(), feature);
            assert_eq!(receipt.input_len(), 3);
            assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
            for input in 0..3 {
                assert_eq!(
                    receipt.input(input).unwrap().outcome(),
                    WidthInputOutcome::NoAction
                );
            }
        } else {
            assert!(output.width_feature_receipt().unwrap().is_none());
        }
    }
}

#[test]
fn retained_zero_space_keeps_its_semantic_identity_without_borrowing_neighbor_geometry() {
    let font = parent_font();
    zero_space_provider_premises(&font);
    let (processed, analyzed) = word_spacing::captured("A  A", WhiteSpace::Collapse, 4.0);
    assert_eq!(processed.source().text(), &Text::from("A  A"));
    assert_eq!(processed.processed_text(), &Text::from("A A"));
    let source_cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(analyzed.combined_cells().len(), 1);
    assert_eq!(
        (
            source_cell.selection_unit_count(),
            source_cell.used_unit_count()
        ),
        (4, 4)
    );
    assert_eq!(
        source_cell.used_code_points().collect::<Vec<_>>(),
        [0x41, 0x20, 0x20, 0x41]
    );
    assert_eq!(
        source_cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x20, 0x41]
    );
    let ancestry = source_cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(ancestry.len(), 4);
    for (index, piece) in ancestry.iter().enumerate() {
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert!(piece.is_used());
        assert_eq!(piece.has_presentation_emission(), index != 2);
    }
    assert_eq!(ancestry[2].emission_subposition(), None);
    assert_eq!(
        processed
            .captured_scope(1)
            .unwrap()
            .style()
            .unwrap()
            .word_spacing,
        4.0
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let origins = (0..4)
        .map(|index| issued_range_origin(&prepared, &processed, index))
        .collect::<Vec<_>>();
    let line = finalize(&prepared);
    one_em(&line);
    let raw = line
        .run_views()
        .flat_map(|run| {
            assert!(run.cell().is_some());
            assert_eq!(run.face().bytes(), font.as_slice());
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        run.origin().x + glyph.pen(),
                        glyph.advance(),
                        glyph.x(),
                        glyph.y(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    // Material SPACE alone gets WS4 halves2+2: normal30 becomes occupied34.
    // The retained zero is neither a fourth provider input nor another WS term.
    assert_eq!(
        raw,
        [
            (35, 0.0, 10.0, 0.0, 0.0),
            (3, 12.0, 10.0, 0.0, 0.0),
            (35, 24.0, 10.0, 0.0, 0.0)
        ]
    );
    let layout = placed_vertical_cell(&prepared, line);
    let results = origins
        .iter()
        .enumerate()
        .map(|(index, origin)| {
            layout
                .source_range_fragments(origin, Range::new(index, index + 1))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let units = results
        .iter()
        .map(|result| {
            assert_eq!(result.units().len(), 1);
            result.units().next().unwrap()
        })
        .collect::<Vec<_>>();
    for (index, unit) in units.iter().copied().enumerate() {
        exact_original(unit, &origins[index], Range::new(index, index + 1));
        let owner = unit.cell_unit().unwrap();
        assert_eq!(
            owner.kind(),
            SourceRangeCellUnitKind::Semantic { ordinal: index }
        );
        assert_eq!(owner.source_cell().used_unit_count(), 4);
        assert_eq!(
            owner.source_cell().scope(),
            processed.captured_scope(1).unwrap()
        );
    }
    let zero = units[2];
    assert_eq!(zero.pieces().len(), 0);
    assert_eq!(
        zero.presentation_range().start(),
        zero.presentation_range().end()
    );
    // No nearest-cut guess: this test requires the actual zero interval, not a
    // fabricated neighboring scalar or a prescribed provider-derived cut number.
    assert!(!zero.same_unit(&units[1]));
    assert!(!zero.same_unit(&units[3]));
    assert!(
        !zero
            .cell_unit()
            .unwrap()
            .same_unit(&units[1].cell_unit().unwrap())
    );
    assert!(units[1].pieces().len() > 0);
    for (index, edges) in [(0, (0.0, 10.0)), (3, (24.0, 34.0))] {
        let pieces = units[index].pieces().collect::<Vec<_>>();
        let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
            panic!("actual visible neighboring A keeps its own font geometry");
        };
        assert_eq!(piece.inner_advance_edges(), edges);
        assert_eq!(piece.effective_instance().face().bytes(), font.as_slice());
        assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
        assert_eq!(piece.metric_frame().origin(), Point::new(95.0, 28.0));
        close(piece.metric_frame().x_scale(), 5.0 / 17.0);
        let rect = piece.rect();
        close(rect.origin.x, 95.0 + edges.0 * (5.0 / 17.0));
        assert_eq!(rect.origin.y, 20.0);
        close(rect.size.width, 50.0 / 17.0);
        assert_eq!(rect.size.height, 10.0);
    }
    drop(system);
    let repeated = layout
        .source_range_fragments(&origins[2], Range::new(2, 3))
        .unwrap();
    assert!(zero.same_unit(&repeated.units().next().unwrap()));
}
