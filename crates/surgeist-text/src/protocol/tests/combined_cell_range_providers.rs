//! Actual optional ligature partitions and distinct fallback font pieces.
use super::*;
use crate::engine::layout::data::{ClusterContent, FinalizedRunFrame};
use crate::protocol::tests::tracking::fixtures::ligature_font;
use crate::{SourceRangeCellUnitKind, SourceRangeGeometry};
use skrifa::{GlyphId, MetadataProvider, string::StringId};
use surgeist_shaper::{
    ComponentCoverage, Direction as ShapeDirection, Feature, FeatureRole, FontRef as ShapeFont,
    ShaperData, ShapingStatus, Tag, UnicodeBuffer, WidthFeature, WidthInputOutcome,
    WidthObservationCoverage, script,
};

pub(in crate::protocol::tests::combined_cells) fn qualified_optional_liga() -> Vec<u8> {
    // The maintained builder independently checks real DFLT/GSUB4 fi35/36->58,
    // source cmap coverage, licensed Ahem outlines and1000-unit target advance.
    let original = ligature_font(*b"liga");
    let bytes = native_metric_font(&original);
    for tag in [
        *b"name", *b"cmap", *b"GSUB", *b"glyf", *b"loca", *b"maxp", *b"hhea", *b"hmtx",
    ] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&original, tag));
    }
    glyph_frame_font_premises(&bytes);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.charmap().map('f'), Some(GlyphId::new(35)));
    assert_eq!(parsed.charmap().map('i'), Some(GlyphId::new(36)));
    assert_eq!(sfnt::u16_at(sfnt::table(&bytes, *b"hmtx"), 58 * 4), 1000);
    let font = ShapeFont::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for enabled in [false, true] {
        let mut input = UnicodeBuffer::new();
        input.add('f', 0);
        input.add('i', 1);
        input.set_direction(ShapeDirection::LeftToRight);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        input.observe_components();
        let features = if enabled {
            vec![]
        } else {
            vec![Feature::new(Tag::new(b"liga"), 0, ..)]
        };
        let output = shaper.shape(input, &features);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .zip(output.glyph_positions())
                .map(|(info, pos)| (
                    info.glyph_id,
                    info.cluster,
                    pos.x_advance,
                    pos.y_advance,
                    pos.x_offset,
                    pos.y_offset
                ))
                .collect::<Vec<_>>(),
            if enabled {
                vec![(58, 0, 1000, 0, 0, 0)]
            } else {
                vec![(35, 0, 1000, 0, 0, 0), (36, 1, 1000, 0, 0, 0)]
            }
        );
        let receipt = output.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.input_len(), 2);
        let ligatures = receipt.ligatures().collect::<Vec<_>>();
        if enabled {
            let [ligature] = ligatures.as_slice() else {
                panic!("one genuine optional liga action");
            };
            assert_eq!(receipt.components(0), Some(&[0, 1][..]));
            assert_eq!(ligature.components(), [0, 1]);
            assert_eq!(ligature.glyph_indices(), [0]);
            assert!(
                ligature
                    .provenance()
                    .iter()
                    .any(|feature| feature.tag() == Tag::new(b"liga")
                        && feature.role() == FeatureRole::Optional)
            );
        } else {
            assert!(ligatures.is_empty());
            assert_eq!(receipt.components(0), Some(&[0][..]));
            assert_eq!(receipt.components(1), Some(&[1][..]));
        }
    }
    bytes
}

#[test]
fn one_actual_optional_liga_glyph_keeps_two_semantic_units_and_the_retained_logical_partition() {
    let bytes = qualified_optional_liga();
    let capture = captured(
        "fi",
        TabSize::spaces(2.0).unwrap(),
        10.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::All,
        LineOrientation::Vertical,
        "Ahem",
    );
    source_premises(&capture, "fi", ParagraphBaseDirection::LeftToRight, &[0, 0]);
    assert!(
        capture
            .processed
            .captured_scope(1)
            .unwrap()
            .style()
            .unwrap()
            .font
            .features
            .is_empty()
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(
            &capture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    container_premises(&prepared, &capture, &bytes);
    let line = complete(&prepared, TabContext::NoTabs);
    assert_eq!(line.feedback().advance, 10.0);
    let actual = line
        .run_views()
        .flat_map(|run| {
            assert!(run.cell().is_some());
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        glyph.range(),
                        run.origin().x + glyph.pen(),
                        glyph.advance(),
                        glyph.x(),
                        glyph.y(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, [(58, Range::new(0, 2), 0.0, 10.0, 0.0, 0.0)]);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.bounds(), Rect::new(95.0, 20.0, 10.0, 10.0));
    let packets = layout.glyph_runs();
    let [packet] = packets.as_slice() else {
        panic!("one actual provider packet");
    };
    assert_eq!(packet.font().bytes(), bytes.as_slice());
    assert_eq!(packet.glyphs().len(), 1);
    assert_eq!(packet.glyph_frame().origin(), Point::new(95.0, 28.0));
    assert_eq!(packet.glyph_frame().outline_x(), Point::new(1.0, 0.0));
    assert_eq!(packet.glyph_frame().outline_y(), Point::new(0.0, 1.0));

    // Actual finalized producer data is qualified BEFORE Range, not obtained
    // from a Range answer. Native extraction gives each of these two eligible
    // ASCII components10/2=5, preserving the final remainder on the last one.
    // This already-retained partition is the authority for the expectations.
    let data = &layout.inner.lines[0].finalized.data;
    assert_eq!(data.cells.len(), 1);
    assert!(data.cells[0].tabs.is_empty());
    let mut indices = [0; 2];
    for (ordinal, index) in indices.iter_mut().enumerate() {
        let rows = data
            .clusters
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row.frame == FinalizedRunFrame::Cell(0)
                    && row.source == Range::new(ordinal, ordinal + 1)
            })
            .collect::<Vec<_>>();
        let [(actual, row)] = rows.as_slice() else {
            panic!("one authentic retained component row");
        };
        assert!(matches!(row.content, ClusterContent::Provider(_)));
        assert_eq!(row.provider_atoms, 0..2); // Actual common provider group.
        assert_eq!(row.shaped_advance, 5.0);
        assert_eq!(row.advance, 5.0);
        assert_eq!(row.x, if ordinal == 0 { 0.0 } else { 5.0 });
        assert_eq!(row.logical_adjustments, [0.0; 3]);
        *index = *actual;
    }
    assert_ne!(indices[0], indices[1]);
    for run in &data.font_runs {
        assert_eq!(run.metrics.font_origin_offset(), 0.0);
    }
    let origins = (0..2)
        .map(|ordinal| {
            let position = capture.processed.captured_origin(ordinal).unwrap();
            let view = prepared
                .origin_views()
                .find(|view| view.capture_position().as_ref() == Some(&position))
                .unwrap();
            assert_eq!(view.local_range(), Range::new(ordinal, ordinal + 1));
            assert_eq!(view.aggregate_range(), Range::new(ordinal, ordinal + 1));
            view.handle()
        })
        .collect::<Vec<_>>();
    assert_ne!(origins[0], origins[1]);
    // Each query is restricted to its OWN issued origin. Shared glyph ancestry
    // does not broaden that query or merge the two Source semantic EGCs.
    let first = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let second = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    assert_eq!((first.units().count(), second.units().count()), (1, 1));
    let a = first.units().next().unwrap();
    let b = second.units().next().unwrap();
    assert!(!a.same_unit(&b));
    assert!(!a.cell_unit().unwrap().same_unit(&b.cell_unit().unwrap()));
    for (ordinal, unit) in [a, b].into_iter().enumerate() {
        assert_eq!(
            unit.cell_unit().unwrap().kind(),
            SourceRangeCellUnitKind::Semantic { ordinal }
        );
        for parts in [
            unit.selected_intersections().collect::<Vec<_>>(),
            unit.expanded_intersections().collect::<Vec<_>>(),
        ] {
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].origin(), origins[ordinal]);
            assert_eq!(parts[0].local_range(), Range::new(ordinal, ordinal + 1));
            assert_eq!(parts[0].aggregate_range(), Range::new(ordinal, ordinal + 1));
        }
        let pieces = unit.pieces().collect::<Vec<_>>();
        let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
            panic!("actual font-backed semantic share");
        };
        assert!(piece.cell_unit().same_unit(&unit.cell_unit().unwrap()));
        assert_eq!(
            piece.inner_advance_edges(),
            if ordinal == 0 {
                (0.0, 5.0)
            } else {
                (5.0, 10.0)
            }
        );
        // Getter ordinals are actual LINE finalized rows, not invented glyph or
        // EGC ordinals. Shared-group views may retain both real contributor rows.
        let contributors = piece.inner_cluster_indices().collect::<Vec<_>>();
        assert!(contributors.contains(&indices[ordinal]));
        assert!(contributors.iter().all(|index| indices.contains(index)));
        assert!(
            piece
                .effective_instance()
                .face()
                .is_same_face(&packet.font().face())
        );
        assert_eq!(piece.effective_instance().font_size(), 10.0);
        assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
        assert_eq!(piece.metric_frame().origin(), Point::new(95.0, 28.0));
        assert_eq!(piece.metric_frame().x_scale(), 1.0); // Real ligature already fits.
        assert_eq!(
            piece.rect(),
            if ordinal == 0 {
                Rect::new(95.0, 20.0, 5.0, 10.0)
            } else {
                Rect::new(100.0, 20.0, 5.0, 10.0)
            }
        );
    }
}

fn family(bytes: &[u8]) -> String {
    skrifa::FontRef::new(bytes)
        .unwrap()
        .localized_strings(StringId::FAMILY_NAME)
        .english_or_first()
        .unwrap()
        .to_string()
}

fn genuine_fallback_faces() -> [Vec<u8>; 2] {
    // Exact maintained src/tests/fallback_coverage.rs::fonts recipe: Ahem
    // kana resource lacks X; real mark resource is renamed Bhem, not aliased.
    let primary = crate::shaping_test_fonts::kana_ahem();
    let secondary = crate::shaping_test_fonts::mark_ahem();
    let mut name = sfnt::table(&secondary, *b"name").to_vec();
    for at in 0..name.len().saturating_sub(7) {
        if &name[at..at + 8] == b"\0A\0h\0e\0m" {
            name[at + 1] = b'B';
        }
    }
    let secondary = sfnt::replace_tables(&secondary, vec![(*b"name", name)]);
    let mut native = Vec::new();
    for (original, expected_family) in [(primary, "Ahem"), (secondary, "Bhem")] {
        let bytes = native_metric_font(&original);
        for tag in [
            *b"name", *b"cmap", *b"glyf", *b"loca", *b"maxp", *b"hhea", *b"hmtx",
        ] {
            assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&original, tag));
        }
        glyph_frame_font_premises(&bytes);
        assert_eq!(family(&bytes), expected_family);
        native.push(bytes);
    }
    let [primary, secondary]: [Vec<u8>; 2] = native.try_into().unwrap();
    let a = skrifa::FontRef::new(&primary).unwrap();
    let b = skrifa::FontRef::new(&secondary).unwrap();
    assert_eq!(a.charmap().map('A'), Some(GlyphId::new(35)));
    assert!(a.charmap().map('X').is_none());
    assert_eq!(b.charmap().map('X'), Some(GlyphId::new(58)));
    for (bytes, ch, glyph) in [(&primary, 'A', 35), (&secondary, 'X', 58)] {
        assert_eq!(
            sfnt::u16_at(sfnt::table(bytes, *b"hmtx"), glyph as usize * 4),
            1000
        );
        let font = ShapeFont::new(bytes).unwrap();
        let data = ShaperData::new(&font);
        let shaper = data.shaper(&font).build();
        assert_eq!(shaper.units_per_em(), 1000);
        for width in [
            None,
            Some((WidthFeature::HalfWidth, *b"hwid")),
            Some((WidthFeature::ThirdWidth, *b"twid")),
            Some((WidthFeature::QuarterWidth, *b"qwid")),
        ] {
            let mut input = UnicodeBuffer::new();
            input.add(ch, 0);
            input.set_direction(ShapeDirection::LeftToRight);
            input.set_script(script::LATIN);
            input.set_language("en".parse().unwrap());
            let features = if let Some((feature, tag)) = width {
                input.observe_width_feature(feature);
                vec![Feature::new(Tag::new(&tag), 1, ..)]
            } else {
                vec![]
            };
            let result = shaper.shape(input, &features);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(
                result
                    .glyph_infos()
                    .iter()
                    .zip(result.glyph_positions())
                    .map(|(info, pos)| (
                        info.glyph_id,
                        info.cluster,
                        pos.x_advance,
                        pos.y_advance,
                        pos.x_offset,
                        pos.y_offset
                    ))
                    .collect::<Vec<_>>(),
                [(glyph, 0, 1000, 0, 0, 0)]
            );
            if let Some((feature, _)) = width {
                let receipt = result.width_feature_receipt().unwrap().unwrap();
                assert_eq!(receipt.feature(), feature);
                assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
                assert_eq!(receipt.input_len(), 1);
                assert_eq!(
                    receipt.input(0).unwrap().outcome(),
                    WidthInputOutcome::NoAction
                );
            } else {
                assert!(result.width_feature_receipt().unwrap().is_none());
            }
        }
    }
    [primary, secondary]
}

#[test]
fn a_single_real_cell_keeps_two_actual_fallback_faces_and_each_own_metric_piece() {
    let bytes = genuine_fallback_faces();
    let body_font = Font::new().family("Ahem").family("Bhem");
    assert_eq!(body_font.family, ["Ahem", "Bhem"]);
    let capture = captured_with_limits(
        Case {
            text: "AX",
            size: TabSize::spaces(2.0).unwrap(),
            root_size: 20.0,
            orientation: LineOrientation::Vertical,
            inner: ParagraphBaseDirection::LeftToRight,
        },
        TextCombineUpright::All,
        body_font,
        ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216),
    );
    source_premises(&capture, "AX", ParagraphBaseDirection::LeftToRight, &[0, 0]);
    assert_eq!(
        capture
            .processed
            .captured_scope(1)
            .unwrap()
            .style()
            .unwrap()
            .font
            .family,
        ["Ahem", "Bhem"]
    );
    assert_eq!(
        capture.processed.source().spans()[0].style().font.family,
        ["Ahem", "Bhem"]
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    for face in &bytes {
        system.register_fonts(face.clone()).unwrap();
    }
    assert_eq!(system.registered_font_faces().count(), 2);
    let prepared = system
        .prepare(
            &capture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    // True Inline container metrics are still the first actual font Ahem10;
    // fallback paint for X cannot donate a second/wrong container metric row.
    container_premises(&prepared, &capture, &bytes[0]);
    assert_eq!(prepared.cell_views().len(), 1);
    let line = complete(&prepared, TabContext::NoTabs);
    assert_eq!(line.cell_views().len(), 1);
    assert_eq!(line.feedback().advance, 10.0);
    let actual = line
        .run_views()
        .map(|run| {
            assert!(run.cell().is_some());
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            let expected = if run.range() == Range::new(0, 1) {
                0
            } else {
                assert_eq!(run.range(), Range::new(1, 2));
                1
            };
            assert_eq!(run.face().bytes(), bytes[expected].as_slice());
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.font_size(), 10.0);
            let metrics = instance
                .metrics(instance.face().catalogue_identity())
                .unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
            // Native wrapper BASE: Latn VerticalX ideo0/idtp1200 at UPEM1000,
            // so the real named Central coordinate is600*10/1000=6 on EACH face.
            let reference = reference(LineOrientation::Vertical);
            let baseline = instance
                .baseline(
                    instance.face().catalogue_identity(),
                    reference.script,
                    reference.axis,
                    reference.baseline,
                )
                .unwrap();
            let FontBaselineOutcome::Available(value) = baseline.outcome() else {
                panic!("actual native Central reference");
            };
            assert_eq!(value.coordinate(), 6.0);
            let glyphs = run
                .glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        glyph.range(),
                        run.origin().x + glyph.pen(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>();
            (expected, glyphs)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (0, vec![(35, Range::new(0, 1), 0.0, 10.0)]),
            (1, vec![(58, Range::new(1, 2), 10.0, 10.0)])
        ]
    );
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.bounds(), Rect::new(95.0, 20.0, 10.0, 10.0));
    let data = &layout.inner.lines[0].finalized.data;
    assert_eq!(data.cells.len(), 1);
    for row in &data.font_runs {
        assert_eq!(row.metrics.font_origin_offset(), 0.0);
    }
    let mut origins = Vec::new();
    for ordinal in 0..2 {
        let position = capture.processed.captured_origin(ordinal).unwrap();
        let view = prepared
            .origin_views()
            .find(|view| view.capture_position().as_ref() == Some(&position))
            .unwrap();
        assert_eq!(view.local_range(), Range::new(ordinal, ordinal + 1));
        assert_eq!(view.aggregate_range(), Range::new(ordinal, ordinal + 1));
        origins.push(view.handle());
    }
    let a = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let x = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    assert_eq!((a.units().count(), x.units().count()), (1, 1));
    assert!(
        !a.units()
            .next()
            .unwrap()
            .same_unit(&x.units().next().unwrap())
    );
    for (ordinal, result) in [&a, &x].into_iter().enumerate() {
        let unit = result.units().next().unwrap();
        let owner = unit.cell_unit().unwrap();
        assert_eq!(owner.kind(), SourceRangeCellUnitKind::Semantic { ordinal });
        assert_eq!(owner.cell_index(), 0);
        assert_eq!(
            owner.source_cell().scope(),
            capture.processed.captured_scope(1).unwrap()
        );
        for pieces in [
            unit.selected_intersections().collect::<Vec<_>>(),
            unit.expanded_intersections().collect::<Vec<_>>(),
        ] {
            assert_eq!(pieces.len(), 1);
            assert_eq!(pieces[0].origin(), origins[ordinal]);
            assert_eq!(pieces[0].local_range(), Range::new(ordinal, ordinal + 1));
            assert_eq!(
                pieces[0].aggregate_range(),
                Range::new(ordinal, ordinal + 1)
            );
        }
        let pieces = unit.pieces().collect::<Vec<_>>();
        let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
            panic!("the EGC's own actual font piece");
        };
        assert!(piece.cell_unit().same_unit(&owner));
        assert_eq!(
            piece.effective_instance().face().bytes(),
            bytes[ordinal].as_slice()
        );
        assert_eq!(
            family(piece.effective_instance().face().bytes()),
            if ordinal == 0 { "Ahem" } else { "Bhem" }
        );
        assert_eq!(piece.effective_instance().font_size(), 10.0);
        assert_eq!(
            piece.inner_advance_edges(),
            if ordinal == 0 {
                (0.0, 10.0)
            } else {
                (10.0, 20.0)
            }
        );
        assert_eq!(piece.metric_frame().origin(), Point::new(95.0, 28.0));
        assert_eq!(piece.metric_frame().x_scale(), 0.5);
        assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
        assert_eq!(
            piece.rect(),
            if ordinal == 0 {
                Rect::new(95.0, 20.0, 5.0, 10.0)
            } else {
                Rect::new(100.0, 20.0, 5.0, 10.0)
            }
        );
    }
}
