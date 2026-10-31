//! Range units follow actual Cell-owned original pieces and finalized
//! Cell ordinals, including partial queries and provider-empty owned source.

use super::*;
use skrifa::MetadataProvider;

fn digit_font() -> Vec<u8> {
    let bytes = qualified_font();
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    for (ch, glyph) in [('1', 19), ('2', 20)] {
        assert_eq!(parsed.charmap().map(ch), Some(GlyphId::new(glyph)));
        assert_eq!(
            sfnt::u16_at(
                sfnt::table(&bytes, *b"hmtx"),
                usize::try_from(glyph).unwrap() * 4
            ),
            1000
        );
    }
    let font = ShaperFontRef::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for request in [
        None,
        Some((*b"hwid", WidthFeature::HalfWidth)),
        Some((*b"twid", WidthFeature::ThirdWidth)),
        Some((*b"qwid", WidthFeature::QuarterWidth)),
    ] {
        let mut input = UnicodeBuffer::new();
        input.add('1', 0);
        input.add('2', 1);
        input.set_direction(Direction::LeftToRight);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        let features = if let Some((tag, feature)) = request {
            input.observe_width_feature(feature);
            vec![Feature::new(Tag::new(&tag), 1, ..)]
        } else {
            vec![]
        };
        let buffer = shaper.shape(input, &features);
        assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            buffer
                .glyph_infos()
                .iter()
                .map(|info| (info.glyph_id, info.cluster))
                .collect::<Vec<_>>(),
            [(19, 0), (20, 1)]
        );
        assert_eq!(
            buffer
                .glyph_positions()
                .iter()
                .map(|position| (
                    position.x_advance,
                    position.y_advance,
                    position.x_offset,
                    position.y_offset
                ))
                .collect::<Vec<_>>(),
            [(1000, 0, 0, 0); 2]
        );
        if let Some((_, feature)) = request {
            let receipt = buffer.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.feature(), feature);
            assert_eq!(receipt.input_len(), 2);
            assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
            for ordinal in 0..2 {
                assert_eq!(
                    receipt.input(ordinal).unwrap().outcome(),
                    WidthInputOutcome::NoAction
                );
            }
            assert!(receipt.input(2).is_none());
        } else {
            assert!(buffer.width_feature_receipt().unwrap().is_none());
        }
    }
    bytes
}

fn digits_source() -> (Arc<ProcessedSource>, AnalyzedSource) {
    let extent = Range::new(0, 4);
    let own = style(
        WhiteSpace::Preserve,
        0.0,
        TextCombineUpright::Digits(TextCombineUprightDigits::Two),
    );
    let mut source = Source::identified("A12B", Id::from_u64(784_967), 1);
    source.span(extent, own.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: extent,
                aggregate: extent,
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![SourceProcessingScope::Block {
                topology: SourceProcessingTopology::Complete,
                trim: WhiteSpaceTrim::default(),
                base_style: own,
                used_direction: ParagraphBaseDirection::LeftToRight,
                writing_axis: Some(SourceWritingAxis::Vertical),
                horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                font_reference: Some(font_reference()),
            }],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("A12B"));
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::Mixed,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let cells = analyzed.combined_cells().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("only the actual bounded ASCII digit sequence is a Cell");
    };
    assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
    assert_eq!(cell.parent_scope(), None); // Genuine derived RootInline, not a fabricated child.
    assert_eq!(
        cell.mode(),
        TextCombineUpright::Digits(TextCombineUprightDigits::Two)
    );
    assert_eq!(cell.selection_unit_count(), 2);
    assert_eq!(cell.used_unit_count(), 2);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x31, 0x32]);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 2);
    for (piece, range) in pieces.iter().zip([Range::new(1, 2), Range::new(2, 3)]) {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), range);
        assert_eq!(piece.aggregate(), range);
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(0).unwrap()
        );
        assert!(piece.is_used() && piece.has_presentation_emission());
    }
    (processed, analyzed)
}

fn exact_origin(prepared: &PreparedText, processed: &ProcessedSource) -> PreparedOrigin {
    assert_eq!(prepared.origin_views().len(), 1);
    let capture = processed.captured_origin(0).unwrap();
    prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&capture))
        .unwrap()
        .handle()
}

fn ordinary(layout: &Layout, origin: &PreparedOrigin, range: Range) {
    let result = layout
        .source_range_fragments(origin, range)
        .expect("ordinary subrange of the SAME original remains observable");
    let units = result.units().collect::<Vec<_>>();
    assert!(!units.is_empty());
    let selected = units
        .iter()
        .flat_map(|unit| {
            unit.selected_intersections()
                .map(|piece| {
                    assert_eq!(
                        piece.origin().view().capture_position(),
                        origin.view().capture_position()
                    );
                    (piece.local_range(), piece.aggregate_range())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(selected, [(range, range)]);
    assert!(units.iter().any(|unit| unit.pieces().len() > 0));
}

#[test]
fn one_real_root_original_distinguishes_ordinary_partial_and_crossing_cell_queries() {
    let font = digit_font();
    let (processed, analyzed) = digits_source();
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
    let origin = exact_origin(&prepared, &processed);
    assert_eq!(origin.view().source(), processed.source());
    assert_eq!(origin.view().local_range(), Range::new(0, 4));
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(0).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        font.as_slice()
    );
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual Digits2 Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    let line = complete_line(&prepared);
    assert_eq!(line.feedback().advance, 30.0);
    let final_cells = line.cell_views().collect::<Vec<_>>();
    let [final_cell] = final_cells.as_slice() else {
        panic!("one actual finalized RootInline cell");
    };
    assert_eq!(
        final_cell.source_cell().scope(),
        processed.captured_scope(0).unwrap()
    );
    assert_eq!(final_cell.square().origin.x, 10.0);
    assert_eq!(final_cell.square().size, Size::new(10.0, 10.0));
    let actual = line
        .run_views()
        .filter(|run| run.cell().is_some())
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), font.as_slice());
            run.glyphs()
                .map(|glyph| (glyph.id(), run.origin().x + glyph.pen(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, [(19, 0.0, 10.0), (20, 10.0, 10.0)]);
    drop(system);
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), processed.source());
    ordinary(&layout, &origin, Range::new(0, 1));
    ordinary(&layout, &origin, Range::new(3, 4));
    // Normal two10px digits have W20 and complete negative width actions;
    // compression .5 preserves two separate source EGCs inside the E10 square.
    let partial = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    assert_eq!(partial.origin(), &origin);
    assert_eq!(partial.local_range(), Range::new(1, 2));
    let units = partial.units().collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        panic!("digit1 is one semantic EGC");
    };
    let cell = unit.cell_unit().unwrap();
    assert_eq!((cell.line_index(), cell.cell_index()), (0, 0));
    assert_eq!(
        cell.kind(),
        SourceRangeCellUnitKind::Semantic { ordinal: 0 }
    );
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(0).unwrap()
    );
    for intersections in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        let [piece] = intersections.as_slice() else {
            panic!("one genuine digit original contribution");
        };
        assert_eq!(piece.origin(), origin);
        assert_eq!(piece.local_range(), Range::new(1, 2));
        assert_eq!(piece.aggregate_range(), Range::new(1, 2));
    }
    let pieces = unit.pieces().collect::<Vec<_>>();
    let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
        panic!("digit1 keeps horizontal font geometry");
    };
    assert!(piece.cell_unit().same_unit(&cell));
    assert_eq!(piece.inner_advance_edges(), (0.0, 10.0));
    assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
    assert_eq!(piece.effective_instance().face().bytes(), font.as_slice());
    assert_eq!(piece.metric_frame().x_scale(), 0.5);
    assert_eq!(piece.rect(), Rect::new(95.0, 30.0, 5.0, 10.0));
    let crossing = layout
        .source_range_fragments(&origin, Range::new(0, 2))
        .unwrap();
    assert_eq!(crossing.origin(), &origin);
    assert_eq!(crossing.local_range(), Range::new(0, 2));
    let crossed = crossing.units().collect::<Vec<_>>();
    assert_eq!(crossed.len(), 2);
    assert!(crossed[0].cell_unit().is_none());
    assert!(crossed[0].pieces().len() > 0);
    assert!(
        crossed[0]
            .pieces()
            .all(|piece| matches!(piece, SourceRangeGeometry::Line(_)))
    );
    assert!(crossed[1].same_unit(unit));
    assert_eq!(
        crossed
            .iter()
            .flat_map(|u| u
                .selected_intersections()
                .map(|i| i.local_range())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    ordinary(&layout, &origin, Range::new(0, 1));
}

#[test]
fn provider_empty_cell_keeps_its_authentic_owned_range_without_fabricated_geometry() {
    let font = qualified_font();
    let (processed, analyzed) = captured(" ", WhiteSpace::Collapse, 0.0);
    source_custody(&processed, &analyzed, &[], 0.0);
    let source_cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(source_cell.selection_unit_count(), 1);
    assert_eq!(source_cell.used_unit_count(), 0);
    let pieces = source_cell.source_pieces().collect::<Vec<_>>();
    let [piece] = pieces.as_slice() else {
        panic!("one retained owned SPACE original");
    };
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(0).unwrap()
    );
    assert_eq!(piece.local(), Range::new(0, 1));
    assert!(!piece.is_used() && !piece.has_presentation_emission());
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
    let origin = exact_origin(&prepared, &processed);
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one real source-nonempty provider-empty Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(!cell.has_provider_input());
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        font.as_slice()
    );
    let line = complete_line(&prepared);
    one_em(&line);
    assert_eq!(line.cell_views().len(), 1);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().len())
            .sum::<usize>(),
        0
    );
    drop(system);
    let layout = placed_vertical_cell(&prepared, line);
    let range = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert_eq!(range.origin(), &origin);
    assert_eq!(range.local_range(), Range::new(0, 1));
    let units = range.units().collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        panic!("one source-proved removed SPACE");
    };
    let cell = unit.cell_unit().unwrap();
    assert_eq!((cell.line_index(), cell.cell_index()), (0, 0));
    assert!(matches!(
        cell.kind(),
        SourceRangeCellUnitKind::Removed { .. }
    ));
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    assert_eq!(
        (
            unit.presentation_range().start(),
            unit.presentation_range().end()
        ),
        (0, 0)
    );
    assert_eq!(unit.pieces().len(), 0); // The real one-em box does not invent a font rectangle.
    for intersections in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        let [actual] = intersections.as_slice() else {
            panic!("one original SPACE contribution");
        };
        assert_eq!(actual.origin(), origin);
        assert_eq!(actual.local_range(), Range::new(0, 1));
        assert_eq!(actual.aggregate_range(), Range::new(0, 1));
        assert_eq!(
            actual.origin().view().capture_position(),
            Some(processed.captured_origin(0).unwrap())
        );
    }
    #[cfg(feature = "text-accessibility")]
    {
        use accesskit::NodeId;
        let mut state = Accessibility::new(43_000);
        let parent = NodeId(1);
        analyzed
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        let start = Cursor::new(0, Affinity::After);
        let end = Cursor::new(1, Affinity::Before);
        let selection = Selection::new(start, end);
        let before_start = analyzed
            .access_position(&state, start)
            .unwrap()
            .expect("original SPACE start binding");
        let before_end = analyzed
            .access_position(&state, end)
            .unwrap()
            .expect("original SPACE end binding");
        let before_selection = analyzed
            .access_selection(&state, selection)
            .unwrap()
            .expect("original SPACE selection binding");
        let error = layout
            .accessibility_update(&mut state, parent, parent, Point::new(f64::INFINITY, 0.0))
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
        assert!(matches!(
            error.detail(),
            Some(ErrorDetail::InvalidTextNumeric {
                phase: TextNumericPhase::Accessibility,
                category: TextNumericError::NonFinite,
                ..
            })
        ));
        assert_eq!(
            analyzed.access_position(&state, start).unwrap(),
            Some(before_start)
        );
        assert_eq!(
            analyzed.access_position(&state, end).unwrap(),
            Some(before_end)
        );
        assert_eq!(
            analyzed.access_selection(&state, selection).unwrap(),
            Some(before_selection)
        );
    }
}
