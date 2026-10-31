//! Genuine Source-owned inner bidi controls, consumed before native Cell shaping.
//! Native preparation consumes completed source semantics through public prepare.
use super::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use surgeist_shaper::{
    Direction as ProviderDirection, Feature, FontRef as ProviderFont, ShaperData, ShapingStatus,
    Tag, UnicodeBuffer, WidthFeature, WidthInputOutcome, WidthObservationCoverage, script,
};

#[derive(Clone, Copy)]
enum InputKind {
    CapturedOverride,
    CapturedIsolate,
    LiteralOverride,
}

impl InputKind {
    fn text(self) -> &'static str {
        match self {
            Self::LiteralOverride => "A\u{202e}BC\u{202c}",
            _ => "ABC",
        }
    }
    fn levels(self) -> [u8; 3] {
        match self {
            Self::CapturedIsolate => [0, 2, 0],
            _ => [0, 1, 1],
        }
    }
    fn visual(self) -> [u32; 3] {
        match self {
            Self::CapturedIsolate => [35, 36, 37],
            _ => [35, 37, 36],
        }
    }
}

fn reference(orientation: LineOrientation) -> ScopeFontReference {
    if orientation == LineOrientation::Vertical {
        font_reference()
    } else {
        ScopeFontReference {
            script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            axis: FontBaselineAxis::HorizontalY,
            baseline: FontBaseline::Alphabetic,
        }
    }
}

fn qualified_font() -> Vec<u8> {
    let bytes = parent_font();
    glyph_frame_font_premises(&bytes);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.os2().unwrap().fs_selection().bits() & 0x80, 0);
    assert!(face.fvar().is_err());
    for (ch, glyph) in [('A', 35), ('B', 36), ('C', 37)] {
        assert_eq!(face.charmap().map(ch), Some(GlyphId::new(glyph)));
        assert_eq!(
            sfnt::u16_at(sfnt::table(&bytes, *b"hmtx"), glyph as usize * 4),
            1000
        );
    }
    let font = ProviderFont::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    // Independently qualify normal glyphs and complete negative width actions
    // on this exact final native-table font, in both genuine inner directions.
    for direction in [
        ProviderDirection::LeftToRight,
        ProviderDirection::RightToLeft,
    ] {
        for request in [
            None,
            Some((*b"hwid", WidthFeature::HalfWidth)),
            Some((*b"twid", WidthFeature::ThirdWidth)),
            Some((*b"qwid", WidthFeature::QuarterWidth)),
        ] {
            let mut input = UnicodeBuffer::new();
            for (ordinal, ch) in "ABC".chars().enumerate() {
                input.add(ch, ordinal as u32);
            }
            input.set_direction(direction);
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
            let expected = if direction == ProviderDirection::LeftToRight {
                [(35, 0), (36, 1), (37, 2)]
            } else {
                [(37, 2), (36, 1), (35, 0)]
            };
            assert_eq!(
                output
                    .glyph_infos()
                    .iter()
                    .map(|g| (g.glyph_id, g.cluster))
                    .collect::<Vec<_>>(),
                expected
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
                for ordinal in 0..3 {
                    assert_eq!(
                        receipt.input(ordinal).unwrap().outcome(),
                        WidthInputOutcome::NoAction
                    );
                }
            } else {
                assert!(output.width_feature_receipt().unwrap().is_none());
            }
        }
    }
    bytes
}

fn captured(
    kind: InputKind,
    mode: TextCombineUpright,
    orientation: LineOrientation,
) -> (Arc<ProcessedSource>, AnalyzedSource) {
    let text = kind.text();
    let length = text.encode_utf16().count();
    let own = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        text_combine_upright: mode,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let mut source = Source::identified(text, Id::from_u64(784_989_752), 1);
    source.span(Range::new(0, length), own.clone());
    let controls = match kind {
        // UAX9 X4: an unmatched RLO is valid and is terminated by this inner
        // isolated paragraph's end. No exterior PDF anchor is borrowed into
        // the Cell or accidentally counted as an extra opaque Cell edge.
        InputKind::CapturedOverride => vec![ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::RightToLeftOverride,
        }],
        // Both RLI and PDI are strictly interior (around B). X5a establishes
        // isolate level1; I2 gives its Latin B level2. A/C remain level0.
        InputKind::CapturedIsolate => vec![
            ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::RightToLeftIsolate,
            },
            ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::PopDirectionalIsolate,
            },
        ],
        InputKind::LiteralOverride => vec![],
    };
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for origin in 0..length {
        for (control, _) in controls
            .iter()
            .enumerate()
            .filter(|(_, fact)| fact.at == origin)
        {
            events.push(SourceProcessingEvent::Control {
                control,
                affinity: Affinity::Before,
            });
        }
        if origin != 0 {
            events.push(SourceProcessingEvent::Connection(origin - 1));
        }
        events.push(SourceProcessingEvent::Origin {
            origin,
            affinity: Affinity::After,
        });
    }
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..length)
                .map(|at| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(at, at + 1),
                    aggregate: Range::new(at, at + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..length - 1)
                .map(|before| SourceProcessingConnection {
                    before_origin: before,
                    after_origin: before + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..length - 1)
                .map(|connection| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(connection),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            controls,
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
                font_reference: Some(reference(orientation)),
            }],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), source.text());
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        if orientation == LineOrientation::Vertical {
            AnalysisTypography::Mixed
        } else {
            AnalysisTypography::HorizontalComposition
        },
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    assert_eq!(analyzed.source(), &source);
    assert_eq!(
        analyzed
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    (processed, analyzed)
}

fn selected(prepared: &PreparedText) -> SelectedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    prepared
        .select(LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: prepared
                .participants()
                .filter_map(|participant| {
                    if let PreparedParticipant::Control(occurrence) = participant {
                        Some(ControlAllocation {
                            occurrence,
                            boundary: None,
                            strut: None,
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                alignment_insets: LineAlignmentInsets::ZERO,
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: prepared.orientation(),
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap()
}

fn complete(prepared: &PreparedText) -> FinalizedLine {
    selected(prepared)
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn source_premises(kind: InputKind, processed: &ProcessedSource, analyzed: &AnalyzedSource) {
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(analyzed.combined_cells().len(), 1);
    assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
    assert_eq!(cell.parent_scope(), None);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.outer_bidi_level().get(), 0);
    assert_eq!(
        cell.selection_unit_count(),
        kind.text().encode_utf16().count()
    );
    assert_eq!(cell.used_unit_count(), 3);
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        [0x41, 0x42, 0x43]
    );
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x42, 0x43]
    );
    assert_eq!(
        cell.inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        kind.levels()
    );
    // These are the actual source-completed provider atoms, before prepare.
    // Literal controls retain original parts but are not body/provider scalars.
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|a| a.code_point)
            .collect::<Vec<_>>(),
        [0x41, 0x42, 0x43]
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), kind.text().encode_utf16().count());
    for (index, (piece, ch)) in pieces.iter().zip(kind.text().chars()).enumerate() {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        let used = matches!(ch, 'A' | 'B' | 'C');
        assert_eq!(piece.is_used(), used);
        assert_eq!(piece.has_presentation_emission(), used);
    }
    let expected = match kind {
        InputKind::CapturedOverride => vec![(1, ParagraphControlKind::RightToLeftOverride)],
        InputKind::CapturedIsolate => vec![
            (1, ParagraphControlKind::RightToLeftIsolate),
            (2, ParagraphControlKind::PopDirectionalIsolate),
        ],
        InputKind::LiteralOverride => vec![],
    };
    for (index, (at, kind)) in expected.iter().enumerate() {
        let control = processed.captured_control(index).unwrap();
        assert_eq!(control.kind(), kind);
        assert_eq!(control.source_position().get(), *at);
        assert_eq!(control.scope(), processed.captured_scope(0).unwrap());
        assert_eq!(
            control.paragraph(),
            processed.captured_paragraph(0).unwrap()
        );
        assert!(processed.captured_origin(at - 1).unwrap() < control.event_position());
        assert!(control.event_position() < processed.captured_origin(*at).unwrap());
    }
    assert!(processed.captured_control(expected.len()).is_none());
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

fn qualify(kind: InputKind) {
    let bytes = qualified_font();
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
        // A separately authored None capture of the same actual source/control
        // sequence must prepare successfully before the Cell refusal is tested.
        let (ordinary_source, ordinary) = captured(kind, TextCombineUpright::None, orientation);
        assert_eq!(ordinary.combined_cells().len(), 0);
        let options = PreparationOptions::default().with_orientation(orientation);
        let ordinary_prepared = system.prepare(&ordinary, options).unwrap();
        let ordinary_line = complete(&ordinary_prepared);
        assert_eq!(ordinary_line.feedback().advance, 30.0);
        assert_eq!(ordinary_line.cell_views().len(), 0);
        assert_eq!(
            ordinary_prepared.analyzed_source().source(),
            ordinary_source.source()
        );
        for run in ordinary_line
            .run_views()
            .filter(|run| run.glyphs().next().is_some())
        {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.axis_count(), 0);
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
                panic!("actual Alphabetic zero must be available");
            };
            assert_eq!(value.coordinate(), 0.0);
            assert_eq!(
                value.provenance(),
                FontBaselineProvenance::HorizontalAlphabeticZero
            );
        }

        let (processed, analyzed) = captured(kind, TextCombineUpright::All, orientation);
        source_premises(kind, &processed, &analyzed);
        let prepared = system
            .prepare(&analyzed, options)
            .expect("native Cell preparation must consume its completed inner bidi controls");
        let cell = prepared.cell_views().next().unwrap();
        assert!(cell.has_provider_input());
        let (before, after, square) = match orientation {
            LineOrientation::Vertical => (5.0, 5.0, Rect::new(95.0, 20.0, 10.0, 10.0)),
            LineOrientation::SidewaysClockwise => (8.0, 2.0, Rect::new(98.0, 20.0, 10.0, 10.0)),
            LineOrientation::SidewaysCounterclockwise => {
                (8.0, 2.0, Rect::new(92.0, 10.0, 10.0, 10.0))
            }
            _ => unreachable!(),
        };
        assert_eq!(
            (cell.extent(), cell.before(), cell.after()),
            (10.0, before, after)
        );
        let metrics = prepared
            .scope_metrics(&processed.captured_scope(0).unwrap())
            .unwrap();
        assert_eq!(metrics.font_reference(), reference(orientation));
        assert_eq!(metrics.font_size(), 10.0);
        assert_eq!(
            metrics.effective_instance().unwrap().face().bytes(),
            bytes.as_slice()
        );
        let line = complete(&prepared);
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(line.cell_views().len(), 1);
        let verify = |line: FinalizedLine| {
            let layout = Layout::from_finalized_lines(PlacementRequest {
                prepared: prepared.clone(),
                coverage: PlacementCoverage::WholeParagraph,
                entries: vec![PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 20.0,
                        baseline: 100.0,
                    },
                    orientation,
                    atomics: vec![],
                })],
            })
            .unwrap();
            assert_eq!(layout.bounds(), square);
            assert_eq!(layout.source(), processed.source());
            assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
            let mut glyphs = Vec::new();
            for run in layout.run_views() {
                if run.glyphs().next().is_none() {
                    continue;
                }
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.font_size(), 10.0);
                assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
                let frame = run.glyph_frame();
                assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
                close(frame.outline_x().x, 1.0 / 3.0);
                assert_eq!(frame.outline_x().y, 0.0);
                assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
                close(frame.origin().y, square.origin.y + 8.0);
                for glyph in run.glyphs() {
                    assert_eq!((glyph.advance(), glyph.x(), glyph.y()), (10.0, 0.0, 0.0));
                    let x = frame.origin().x + frame.outline_x().x * glyph.pen();
                    glyphs.push((glyph.id(), x));
                }
            }
            glyphs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            assert_eq!(glyphs.len(), 3); // No provider/control glyph is invented.
            assert_eq!(
                glyphs.iter().map(|g| g.0).collect::<Vec<_>>(),
                kind.visual()
            );
            for (index, (_, x)) in glyphs.iter().enumerate() {
                close(*x, square.origin.x + index as f64 * (10.0 / 3.0));
            }
            // Native raw advance is30; complete-negative width receipts force
            // normal geometric compression1/3 into one fixed E10 outer square.
            let start = Cursor::new(0, Affinity::After);
            let end = Cursor::new(kind.text().encode_utf16().count(), Affinity::Before);
            let positions = layout.positions().collect::<Vec<_>>();
            assert_eq!(
                positions
                    .iter()
                    .map(PlacedPosition::source_cursor)
                    .collect::<Vec<_>>(),
                [start, end]
            );
            let (start_y, end_y, near_start, near_end) =
                if orientation == LineOrientation::SidewaysCounterclockwise {
                    (19.0, 9.0, 19.0, 11.0)
                } else {
                    (20.0, 30.0, 21.0, 29.0)
                };
            for (position, cursor, y) in
                [(&positions[0], start, start_y), (&positions[1], end, end_y)]
            {
                let expected = Rect::new(square.origin.x, y, 10.0, 1.0);
                assert_eq!(layout.position_geometry(position).unwrap().rect(), expected);
                assert_eq!(layout.cursor(cursor).unwrap().rect(), expected);
            }
            for (y, position) in [(near_start, &positions[0]), (near_end, &positions[1])] {
                let Hit::Text(hit) = layout.hit(Point::new(square.origin.x + 5.0, y)) else {
                    panic!("actual Cell square hit");
                };
                assert_eq!(&hit, position);
            }
            assert_eq!(
                layout
                    .move_position(&positions[0], Movement::NextCluster)
                    .unwrap(),
                positions[1]
            );
            let selection = layout
                .position_selection(&positions[0], &positions[1])
                .unwrap();
            assert_eq!(selection.source_text(), processed.source().text());
            assert_eq!(
                selection.geometry(),
                &SelectionGeometry::new(vec![SelectionRect::new(square, 0)])
            );
            let origin = prepared.origin_views().next().unwrap().handle();
            assert_eq!(
                origin.view().capture_position(),
                Some(processed.captured_origin(0).unwrap())
            );
            let range = layout
                .source_range_fragments(&origin, Range::new(0, 1))
                .unwrap();
            assert_eq!(range.origin(), &origin);
            assert_eq!(range.local_range(), Range::new(0, 1));
            let units = range.units().collect::<Vec<_>>();
            let [unit] = units.as_slice() else {
                panic!("A is one semantic EGC");
            };
            let cell_unit = unit.cell_unit().unwrap();
            assert_eq!((cell_unit.line_index(), cell_unit.cell_index()), (0, 0));
            assert_eq!(
                cell_unit.kind(),
                SourceRangeCellUnitKind::Semantic { ordinal: 0 }
            );
            assert_eq!(
                cell_unit.source_cell().scope(),
                processed.captured_scope(0).unwrap()
            );
            for intersections in [
                unit.selected_intersections().collect::<Vec<_>>(),
                unit.expanded_intersections().collect::<Vec<_>>(),
            ] {
                let [piece] = intersections.as_slice() else {
                    panic!("A keeps its original contribution");
                };
                assert_eq!(piece.origin(), origin);
                assert_eq!(piece.local_range(), Range::new(0, 1));
                assert_eq!(piece.aggregate_range(), Range::new(0, 1));
            }
            let pieces = unit.pieces().collect::<Vec<_>>();
            let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
                panic!("A has genuine horizontal font geometry");
            };
            assert!(piece.cell_unit().same_unit(&cell_unit));
            assert_eq!(piece.inner_advance_edges(), (0.0, 10.0));
            assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
            assert_eq!(piece.effective_instance().face().bytes(), bytes.as_slice());
            close(piece.metric_frame().x_scale(), 1.0 / 3.0);
            close(piece.rect().origin.x, square.origin.x);
            assert_eq!(piece.rect().origin.y, square.origin.y);
            close(piece.rect().size.width, 10.0 / 3.0);
            assert_eq!(piece.rect().size.height, 10.0);
            #[cfg(feature = "text-accessibility")]
            {
                use accesskit::NodeId;
                let mut state = Accessibility::new(50_000);
                let parent = NodeId(1);
                analyzed
                    .accessibility_update(&mut state, parent, parent)
                    .unwrap();
                let before = analyzed.access_position(&state, start).unwrap();
                assert!(before.is_some());
                let error = layout
                    .accessibility_update(&mut state, parent, parent, Point::new(0.0, 0.0))
                    .unwrap_err();
                assert_eq!(error.code, ErrorCode::UnsupportedFeature);
                assert_eq!(
                    error.detail(),
                    Some(&ErrorDetail::UnsupportedAccessibilityCell { line: 0, cell: 0 })
                );
                assert_eq!(analyzed.access_position(&state, start).unwrap(), before);
            }
        };
        verify(line.clone());
        drop(system);
        verify(line.with_spacing(SpacingAllocations::None).unwrap());
        source_premises(kind, &processed, &analyzed);
    }
}

#[test]
fn captured_inner_rlo_reverses_bc_without_becoming_a_provider_scalar_or_outer_unit() {
    // UAX9 X4/I1/I2 and L2: A0,B1,C1 paints A,C,B inside the isolated Cell.
    qualify(InputKind::CapturedOverride);
}

#[test]
fn captured_rli_pdi_pair_resolves_latin_b_inside_the_isolate_without_reversing_it() {
    // RLI/PDI surround only B: used A0,B2,C0 remains visual A,B,C.
    qualify(InputKind::CapturedIsolate);
}

#[test]
fn literal_inner_rlo_pdf_keeps_original_parts_but_supplies_only_abc_to_the_provider() {
    qualify(InputKind::LiteralOverride);
}
