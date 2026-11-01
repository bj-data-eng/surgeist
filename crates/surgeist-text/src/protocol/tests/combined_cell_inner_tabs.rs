//! Preserved CSS TABs execute inside the genuine horizontal Cell packet.
//! Ordinary captured CSS stops and real Shaper receipts qualify inputs first.
use super::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use surgeist_shaper::{
    Direction as ProviderDirection, Feature, FontRef as ProviderFont, ShaperData, ShapingStatus,
    Tag, UnicodeBuffer, WidthFeature, WidthInputOutcome, WidthObservationCoverage, script,
};

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

fn glyph_id(bytes: &[u8], ch: char) -> u32 {
    skrifa::FontRef::new(bytes)
        .unwrap()
        .charmap()
        .map(ch)
        .unwrap()
        .to_u32()
}

pub(super) fn qualified_font() -> Vec<u8> {
    let bytes = parent_font();
    glyph_frame_font_premises(&bytes);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.os2().unwrap().fs_selection().bits() & 0x80, 0);
    assert!(face.fvar().is_err());
    assert!(face.mvar().is_err());
    assert_eq!(face.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(face.charmap().map('B'), Some(GlyphId::new(36)));
    for ch in "AB12 0".chars() {
        let id = glyph_id(&bytes, ch);
        assert_eq!(
            sfnt::u16_at(sfnt::table(&bytes, *b"hmtx"), id as usize * 4),
            1000
        );
    }
    // Neither tab-layout SPACE nor half-ch ZERO is an artificial width glyph.
    // All real provider descendants in these cases have complete negative
    // width observations on the exact final native-table font.
    let font = ProviderFont::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for direction in [
        ProviderDirection::LeftToRight,
        ProviderDirection::RightToLeft,
    ] {
        for width in [
            None,
            Some((*b"hwid", WidthFeature::HalfWidth)),
            Some((*b"twid", WidthFeature::ThirdWidth)),
            Some((*b"qwid", WidthFeature::QuarterWidth)),
        ] {
            let mut input = UnicodeBuffer::new();
            for (at, ch) in "AB12 0".chars().enumerate() {
                input.add(ch, at as u32);
            }
            input.set_direction(direction);
            input.set_script(script::LATIN);
            input.set_language("en".parse().unwrap());
            let features = if let Some((tag, feature)) = width {
                input.observe_width_feature(feature);
                vec![Feature::new(Tag::new(&tag), 1, ..)]
            } else {
                vec![]
            };
            let output = shaper.shape(input, &features);
            assert_eq!(output.shaping_status(), ShapingStatus::Complete);
            let mut expected = "AB12 0"
                .chars()
                .enumerate()
                .map(|(at, ch)| (glyph_id(&bytes, ch), at as u32))
                .collect::<Vec<_>>();
            if direction == ProviderDirection::RightToLeft {
                expected.reverse();
            }
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
                [(1000, 0, 0, 0); 6]
            );
            if let Some((_, feature)) = width {
                let receipt = output.width_feature_receipt().unwrap().unwrap();
                assert_eq!(receipt.feature(), feature);
                assert_eq!(receipt.input_len(), 6);
                assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
                for at in 0..6 {
                    assert_eq!(
                        receipt.input(at).unwrap().outcome(),
                        WidthInputOutcome::NoAction
                    );
                }
            } else {
                assert!(output.width_feature_receipt().unwrap().is_none());
            }
        }
    }
    // FontAdvance is a different existing operation: the real font's TAB
    // .notdef is not a CSS-stop glyph or evidence that CSS TAB uses a provider.
    assert!(face.charmap().map('\t').is_none());
    assert_eq!(sfnt::u16_at(sfnt::table(&bytes, *b"hmtx"), 0), 1000);
    let mut input = UnicodeBuffer::new();
    for (at, ch) in "A\tB".chars().enumerate() {
        input.add(ch, at as u32);
    }
    input.set_direction(ProviderDirection::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let output = shaper.shape(input, &[]);
    assert_eq!(output.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        output
            .glyph_infos()
            .iter()
            .map(|g| g.glyph_id)
            .collect::<Vec<_>>(),
        [35, 0, 36]
    );
    assert_eq!(
        output
            .glyph_positions()
            .iter()
            .map(|p| p.x_advance)
            .collect::<Vec<_>>(),
        [1000; 3]
    );
    bytes
}

pub(super) struct Captured {
    pub(super) processed: Arc<ProcessedSource>,
    pub(super) analyzed: AnalyzedSource,
}

pub(super) fn captured(
    text: &str,
    size: TabSize,
    root_size: f32,
    inner: ParagraphBaseDirection,
    mode: TextCombineUpright,
    orientation: LineOrientation,
    body_family: &str,
) -> Captured {
    captured_with_limits(
        Case {
            text,
            size,
            root_size,
            orientation,
            inner,
        },
        mode,
        Font::new().family(body_family),
        ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216),
    )
}

fn captured_with_limits(
    case: Case<'_>,
    mode: TextCombineUpright,
    body_font: Font,
    limits: ScopeAlignmentLimits,
) -> Captured {
    let Case {
        text,
        size,
        root_size,
        orientation,
        inner,
    } = case;
    let length = text.encode_utf16().count();
    assert!(length > 0);
    let root_style = Style {
        font: Font::new().family("Ahem"),
        size: root_size,
        line_height: LineHeight::Absolute(f64::from(root_size)),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        text_combine_upright: TextCombineUpright::None,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        tab_size: size,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let body = Style {
        font: body_font,
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: mode,
        ..root_style.clone()
    };
    let mut source = Source::identified(text, Id::from_u64(784_992_001), 1);
    source.span(Range::new(0, length), body.clone());
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(1),
    ];
    for origin in 0..length {
        if origin > 0 {
            events.push(SourceProcessingEvent::Connection(origin - 1));
        }
        events.push(SourceProcessingEvent::Origin {
            origin,
            affinity: Affinity::After,
        });
    }
    events.extend([
        SourceProcessingEvent::ExitScope(1),
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
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(limits),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: root_style,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(inner),
                    font_reference: Some(reference(orientation)),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(body),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(reference(orientation)),
                    horizontal_inner_direction: Some(inner),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text));
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
    assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
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
    Captured {
        processed,
        analyzed,
    }
}

fn source_premises(capture: &Captured, text: &str, inner: ParagraphBaseDirection, levels: &[u8]) {
    let cell = capture.analyzed.combined_cells().next().unwrap();
    assert_eq!(capture.analyzed.combined_cells().len(), 1);
    assert_eq!(cell.scope(), capture.processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.parent_scope(),
        Some(capture.processed.captured_scope(0).unwrap())
    );
    assert_eq!(cell.inner_direction(), inner);
    assert_eq!(cell.outer_bidi_level().get(), 0);
    assert_eq!(cell.selection_unit_count(), text.encode_utf16().count());
    assert_eq!(cell.used_unit_count(), text.encode_utf16().count());
    let points = text.chars().map(u32::from).collect::<Vec<_>>();
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), points);
    assert_eq!(cell.presentation_code_points().collect::<Vec<_>>(), points);
    assert_eq!(
        cell.inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        levels
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), text.encode_utf16().count());
    for (at, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), capture.processed.source());
        assert_eq!(piece.local(), Range::new(at, at + 1));
        assert_eq!(piece.aggregate(), Range::new(at, at + 1));
        assert_eq!(
            piece.origin_position(),
            capture.processed.captured_origin(at).unwrap()
        );
        assert_eq!(
            piece.paragraph(),
            capture.processed.captured_paragraph(0).unwrap()
        );
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert!(piece.is_used());
        assert!(piece.has_presentation_emission());
    }
}

fn request(prepared: &PreparedText, tabs: TabContext) -> LineSelectionRequest {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    LineSelectionRequest {
        inline_boxes: None,
        segment: segment.clone(),
        start: segment.start(),
        end: segment.end(),
        participants: prepared.participants().collect(),
        termination: PreparedTermination::Ordinary(segment.end_opportunity()),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls: vec![],
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
            tabs,
            spacing: SpacingPolicy::None,
        },
    }
}

fn complete(prepared: &PreparedText, tabs: TabContext) -> FinalizedLine {
    prepared
        .select(request(prepared, tabs))
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn options(orientation: LineOrientation) -> PreparationOptions {
    PreparationOptions::try_new(1.0, TabPreparation::CssStops)
        .unwrap()
        .with_orientation(orientation)
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

fn raw_glyphs(line: &FinalizedLine, bytes: &[u8], is_cell: bool) -> Vec<(u32, f64)> {
    let mut glyphs = vec![];
    for run in line.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        assert_eq!(run.cell().is_some(), is_cell);
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.advance(), glyph.x(), glyph.y()), (10.0, 0.0, 0.0));
            // Public finalized origin is intrinsic horizontal for Cell runs;
            // glyph.pen itself remains run-local and can restart after a TAB.
            glyphs.push((glyph.id(), run.origin().x + glyph.pen()));
        }
    }
    glyphs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    glyphs
}

fn square(orientation: LineOrientation) -> Rect {
    match orientation {
        LineOrientation::Vertical => Rect::new(95.0, 20.0, 10.0, 10.0),
        LineOrientation::SidewaysClockwise => Rect::new(98.0, 20.0, 10.0, 10.0),
        LineOrientation::SidewaysCounterclockwise => Rect::new(92.0, 10.0, 10.0, 10.0),
        _ => unreachable!(),
    }
}

fn observe(
    prepared: &PreparedText,
    line: FinalizedLine,
    capture: &Captured,
    bytes: &[u8],
    raw: &[(char, f64)],
    width: Option<f64>,
) {
    let orientation = prepared.orientation();
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.measurements().fit().advance(), 10.0);
    assert_eq!(line.cell_views().len(), 1);
    assert_eq!(
        line.cell_views().next().unwrap().source_cell().scope(),
        capture.processed.captured_scope(1).unwrap()
    );
    let expected_raw = raw
        .iter()
        .map(|&(ch, pen)| (glyph_id(bytes, ch), pen))
        .collect::<Vec<_>>();
    let mut observed_raw = raw_glyphs(&line, bytes, true);
    if capture
        .analyzed
        .combined_cells()
        .next()
        .unwrap()
        .inner_direction()
        == ParagraphBaseDirection::RightToLeft
    {
        // The right-start grid can retain a signed intrinsic origin; the public
        // origin contract does not require left normalization. Preserve exact
        // relative raw pens and require the absolute physical frame below.
        let first = observed_raw.first().unwrap().1;
        for (_, pen) in &mut observed_raw {
            *pen -= first;
        }
    }
    assert_eq!(observed_raw, expected_raw);
    if raw.is_empty() {
        assert!(
            line.run_views().all(|run| run.cell().is_none()),
            "TAB-only layout has no provider run/baseline"
        );
    }
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
    let square = square(orientation);
    assert_eq!(layout.bounds(), square);
    assert_eq!(layout.source(), capture.processed.source());
    assert_eq!(
        layout.analyzed_source().identity(),
        capture.analyzed.identity()
    );
    let mut physical = vec![];
    for run in layout.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        let frame = run.glyph_frame();
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        let scale = 10.0 / width.unwrap();
        close(frame.outline_x().x, scale);
        assert_eq!(frame.outline_x().y, 0.0);
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        close(frame.origin().y, square.origin.y + 8.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.advance(), glyph.x(), glyph.y()), (10.0, 0.0, 0.0));
            physical.push((glyph.id(), frame.origin().x + scale * glyph.pen()));
        }
    }
    physical.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    assert_eq!(physical.len(), raw.len());
    for ((id, x), &(ch, pen)) in physical.iter().zip(raw) {
        assert_eq!(*id, glyph_id(bytes, ch));
        close(*x, square.origin.x + 10.0 / width.unwrap() * pen);
    }
    let text_len = capture.processed.source().text().code_units().len();
    let start = Cursor::new(0, Affinity::After);
    let end = Cursor::new(text_len, Affinity::Before);
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
    for (position, cursor, y) in [(&positions[0], start, start_y), (&positions[1], end, end_y)] {
        let expected = Rect::new(square.origin.x, y, 10.0, 1.0);
        assert_eq!(layout.position_geometry(position).unwrap().rect(), expected);
        assert_eq!(layout.cursor(cursor).unwrap().rect(), expected);
    }
    for (y, position) in [(near_start, &positions[0]), (near_end, &positions[1])] {
        let Hit::Text(hit) = layout.hit(Point::new(square.origin.x + 5.0, y)) else {
            panic!("Cell hit");
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
    assert_eq!(selection.source_text(), capture.processed.source().text());
    assert_eq!(
        selection.geometry(),
        &SelectionGeometry::new(vec![SelectionRect::new(square, 0)])
    );
    let origin = prepared.origin_views().next().unwrap().handle();
    assert_eq!(
        origin.view().capture_position(),
        Some(capture.processed.captured_origin(0).unwrap())
    );
    let range = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert_eq!(range.origin(), &origin);
    assert_eq!(range.local_range(), Range::new(0, 1));
    let units = range.units().collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        panic!("the first original owns one semantic EGC");
    };
    let cell_unit = unit.cell_unit().unwrap();
    assert_eq!((cell_unit.line_index(), cell_unit.cell_index()), (0, 0));
    assert_eq!(
        cell_unit.kind(),
        SourceRangeCellUnitKind::Semantic { ordinal: 0 }
    );
    assert_eq!(
        cell_unit.source_cell().scope(),
        capture.processed.captured_scope(1).unwrap()
    );
    for intersections in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        let [piece] = intersections.as_slice() else {
            panic!("one actual first-original contribution");
        };
        assert_eq!(piece.origin(), origin);
        assert_eq!(piece.local_range(), Range::new(0, 1));
        assert_eq!(piece.aggregate_range(), Range::new(0, 1));
    }
    let pieces = unit.pieces().collect::<Vec<_>>();
    if capture.processed.source().text().code_units()[0] == 0x09 {
        // The selected original is a native TAB, including numeric-zero TAB+A.
        // Its body may contain fonts; this selected span has no body baseline.
        let [SourceRangeGeometry::CellTab(tab)] = pieces.as_slice() else {
            panic!("native TAB span must not borrow a body font");
        };
        assert!(tab.cell_unit().same_unit(&cell_unit));
        assert_eq!(tab.span_index(), 0);
        let scope = capture.processed.captured_scope(1).unwrap();
        let size = scope.style().unwrap().tab_size;
        if size.is_spaces() {
            assert_eq!(size.value(), 0.0);
        }
        assert!(size.value() == 20.0 || size.value() == 0.0);
        let count = capture.processed.source().text().len() as f64;
        let period = size.value();
        // Literal packet width is qualified above for mixed TAB+A. TAB-only
        // fixtures execute one actual period per source occurrence.
        let total = width.unwrap_or(count * period);
        let scale = if total > 10.0 { 10.0 / total } else { 1.0 };
        let center = (10.0 - total * scale) / 2.0;
        assert_eq!(tab.inner_advance_edges(), (0.0, period));
        assert_eq!(tab.advance(), period);
        assert_eq!(tab.container_metrics().captured_scope(), scope);
        assert_eq!(tab.container_metrics().font_size(), 10.0);
        assert_eq!(
            tab.container_metrics().font_reference(),
            reference(orientation)
        );
        assert_eq!(
            tab.layout_frame().origin(),
            Point::new(square.origin.x + center, square.origin.y)
        );
        close(tab.layout_frame().x_scale(), scale);
        assert_eq!(tab.layout_frame().em_band_extent(), 10.0);
        close(tab.rect().origin.x, square.origin.x + center);
        assert_eq!(tab.rect().origin.y, square.origin.y);
        close(tab.rect().size.width, period * scale);
        assert_eq!(tab.rect().size.height, 10.0);
    } else {
        // The requested source0..1 is A even for the genuine RTL inner case.
        // Literal raw packet positions above give its physical normalized pen;
        // its10px font band is compressed, never rotated with the outer pen.
        let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
            panic!("A keeps real font geometry beside native TABs");
        };
        let pen = raw.iter().find(|(ch, _)| *ch == 'A').unwrap().1;
        let scale = 10.0 / width.unwrap();
        assert!(piece.cell_unit().same_unit(&cell_unit));
        let (start, end) = piece.inner_advance_edges();
        close(end - start, 10.0);
        assert_eq!(piece.effective_instance().face().bytes(), bytes);
        assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
        close(piece.metric_frame().x_scale(), scale);
        close(piece.rect().origin.x, square.origin.x + pen * scale);
        assert_eq!(piece.rect().origin.y, square.origin.y);
        close(piece.rect().size.width, 10.0 * scale);
        assert_eq!(piece.rect().size.height, 10.0);
    }
    #[cfg(feature = "text-accessibility")]
    {
        use accesskit::NodeId;
        let mut state = Accessibility::new(50_000);
        let parent = NodeId(1);
        capture
            .analyzed
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        let before = capture.analyzed.access_position(&state, start).unwrap();
        assert!(before.is_some());
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
            capture.analyzed.access_position(&state, start).unwrap(),
            before
        );
    }
}

fn container_premises(prepared: &PreparedText, capture: &Captured, bytes: &[u8]) {
    let orientation = prepared.orientation();
    let metrics = prepared
        .scope_metrics(&capture.processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(metrics.font_reference(), reference(orientation));
    let instance = metrics.effective_instance().unwrap();
    assert_eq!(instance.face().bytes(), bytes);
    assert_eq!(instance.axis_count(), 0);
    let horizontal = instance
        .metrics(instance.face().catalogue_identity())
        .unwrap();
    assert_eq!(horizontal.line_metric_source(), FontLineMetricSource::Hhea);
    assert_eq!((horizontal.ascent(), horizontal.descent()), (8.0, -2.0));
    let baseline = instance
        .baseline(
            instance.face().catalogue_identity(),
            reference(orientation).script,
            FontBaselineAxis::HorizontalY,
            FontBaseline::Alphabetic,
        )
        .unwrap();
    let FontBaselineOutcome::Available(value) = baseline.outcome() else {
        panic!("real Alphabetic baseline");
    };
    assert_eq!(value.coordinate(), 0.0);
    assert_eq!(
        value.provenance(),
        FontBaselineProvenance::HorizontalAlphabeticZero
    );
}

struct Case<'a> {
    text: &'a str,
    size: TabSize,
    root_size: f32,
    orientation: LineOrientation,
    inner: ParagraphBaseDirection,
}

struct Expected<'a> {
    levels: &'a [u8],
    ordinary_width: f64,
    ordinary_origin5: Option<f64>,
    ordinary_raw: &'a [(char, f64)],
    raw: &'a [(char, f64)],
    width: Option<f64>,
}

fn qualify(case: Case<'_>, expected: Expected<'_>) {
    let Case {
        text,
        size,
        root_size,
        orientation,
        inner,
    } = case;
    let Expected {
        levels,
        ordinary_width,
        ordinary_origin5,
        ordinary_raw,
        raw,
        width,
    } = expected;
    let bytes = qualified_font();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let ordinary = captured(
        text,
        size,
        root_size,
        inner,
        TextCombineUpright::None,
        orientation,
        "Ahem",
    );
    assert_eq!(ordinary.analyzed.combined_cells().len(), 0);
    let ordinary_prepared = system
        .prepare(&ordinary.analyzed, options(orientation))
        .unwrap();
    if !raw.is_empty() {
        let ordinary_line = complete(
            &ordinary_prepared,
            TabContext::CssStops { origin: Some(0.0) },
        );
        assert_eq!(ordinary_line.feedback().advance, ordinary_width);
        assert_eq!(
            raw_glyphs(&ordinary_line, &bytes, false),
            ordinary_raw
                .iter()
                .map(|&(ch, pen)| (glyph_id(&bytes, ch), pen))
                .collect::<Vec<_>>()
        );
        // The exterior operation still consumes its actual late block origin.
        // A +5 origin moves the next positive grid stop by5 in these cases.
        if let Some(expected) = ordinary_origin5 {
            let shifted = complete(
                &ordinary_prepared,
                TabContext::CssStops { origin: Some(5.0) },
            );
            assert_eq!(shifted.feedback().advance, expected);
        }
    }
    let capture = captured(
        text,
        size,
        root_size,
        inner,
        TextCombineUpright::All,
        orientation,
        "Ahem",
    );
    source_premises(&capture, text, inner, levels);
    // Distinct, unchanged FontAdvance remains a genuine provider operation.
    let font_advance = system
        .prepare(
            &capture.analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    assert!(
        font_advance
            .cell_views()
            .next()
            .unwrap()
            .has_provider_input()
    );
    container_premises(&font_advance, &capture, &bytes);
    let font_advance_line = complete(&font_advance, TabContext::FontAdvance);
    assert_eq!(font_advance_line.feedback().advance, 10.0);
    assert_eq!(
        font_advance_line
            .run_views()
            .map(|run| run.glyphs().len())
            .sum::<usize>(),
        text.encode_utf16().count()
    );
    assert_eq!(
        font_advance_line
            .run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|glyph| glyph.advance())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![10.0; text.encode_utf16().count()]
    );

    let prepared = system
        .prepare(&capture.analyzed, options(orientation))
        .expect("preserved inner CSS TAB must execute before Cell width choice/compression");
    assert_eq!(
        prepared.analyzed_source().identity(),
        capture.analyzed.identity()
    );
    let cell = prepared.cell_views().next().unwrap();
    assert_eq!(prepared.cell_views().len(), 1);
    assert_eq!(cell.has_provider_input(), !raw.is_empty());
    let (before, after) = if orientation == LineOrientation::Vertical {
        (5.0, 5.0)
    } else {
        (8.0, 2.0)
    };
    assert_eq!(
        (cell.extent(), cell.before(), cell.after()),
        (10.0, before, after)
    );
    container_premises(&prepared, &capture, &bytes);
    let line = complete(&prepared, TabContext::NoTabs);
    observe(&prepared, line.clone(), &capture, &bytes, raw, width);
    // These are legitimate outer contexts with no exterior TAB demand. Neither
    // a missing origin nor an unrelated late outer origin changes inner stops.
    for origin in [None, Some(700.0)] {
        observe(
            &prepared,
            complete(&prepared, TabContext::CssStops { origin }),
            &capture,
            &bytes,
            raw,
            width,
        );
    }
    drop(system);
    observe(
        &prepared,
        line.with_spacing(SpacingAllocations::None).unwrap(),
        &capture,
        &bytes,
        raw,
        width,
    );
    source_premises(&capture, text, inner, levels);
}

fn grid_ownership(orientation: LineOrientation) {
    for root_size in [20.0, 30.0] {
        // Absolute20 is independent of either font. Numeric2 is40/60 in the
        // ordinary nearest-Block path, but20 in the Cell's real Inline10.
        for (size, ordinary_width) in [
            (TabSize::length(20.0).unwrap(), 30.0),
            (
                TabSize::spaces(2.0).unwrap(),
                if root_size == 20.0 { 50.0 } else { 70.0 },
            ),
        ] {
            qualify(
                Case {
                    text: "A\tB",
                    size,
                    root_size,
                    orientation,
                    inner: ParagraphBaseDirection::LeftToRight,
                },
                Expected {
                    levels: &[0, 0, 0],
                    ordinary_width,
                    ordinary_origin5: Some(ordinary_width + 5.0),
                    ordinary_raw: &[('A', 0.0), ('B', ordinary_width - 10.0)],
                    raw: &[('A', 0.0), ('B', 20.0)],
                    width: Some(30.0),
                },
            );
        }
    }
}

#[test]
fn vertical_inner_tabs_use_inline_reference_not_root_font_or_late_outer_origin() {
    grid_ownership(LineOrientation::Vertical);
}
#[test]
fn clockwise_inner_tabs_keep_the_horizontal_packet_and_inline_grid() {
    grid_ownership(LineOrientation::SidewaysClockwise);
}
#[test]
fn counterclockwise_inner_tabs_keep_the_horizontal_packet_and_inline_grid() {
    grid_ownership(LineOrientation::SidewaysCounterclockwise);
}

#[test]
fn inner_tabs_use_strict_next_stop_one_half_ch_bump_and_one_shared_repeated_grid() {
    // Specified horizontal ch10 gives a half-ch floor5. Equal5 is accepted;
    // gap4 takes exactly one additional14 period. Gap8 atT18 passes Inline
    // half-ch5; wrongly using Root20 half-ch10 would bump to36/raw46.
    // A tab already on20 advances to40.
    for (text, period, b, width) in [
        ("A\tB", 10.0, 20.0, 30.0),
        ("A\tB", 14.0, 28.0, 38.0),
        ("A\tB", 15.0, 15.0, 25.0),
        ("A\tB", 18.0, 18.0, 28.0),
        ("A\t\tB", 20.0, 40.0, 50.0),
        ("A\tB", 0.0, 10.0, 20.0),
    ] {
        qualify(
            Case {
                text,
                size: TabSize::length(period).unwrap(),
                root_size: 20.0,
                orientation: LineOrientation::Vertical,
                inner: ParagraphBaseDirection::LeftToRight,
            },
            Expected {
                levels: &vec![0; text.encode_utf16().count()],
                ordinary_width: width,
                ordinary_origin5: None,
                ordinary_raw: &[('A', 0.0), ('B', b)],
                raw: &[('A', 0.0), ('B', b)],
                width: Some(width),
            },
        );
    }
}

#[test]
fn preserved_tab_only_cells_have_used_source_and_real_square_without_provider_runs() {
    // No public readback exposes tab-only intrinsic span/scale. These assertions
    // qualify genuine used-source/provider absence, outer geometry and replay;
    // exact retained tab spans need the owning private layout qualification.
    for orientation in [
        LineOrientation::Vertical,
        LineOrientation::SidewaysClockwise,
        LineOrientation::SidewaysCounterclockwise,
    ] {
        for (text, period) in [("\t", 20.0), ("\t\t", 20.0), ("\t", 0.0)] {
            qualify(
                Case {
                    text,
                    size: TabSize::length(period).unwrap(),
                    root_size: 20.0,
                    orientation,
                    inner: ParagraphBaseDirection::LeftToRight,
                },
                Expected {
                    levels: &vec![0; text.encode_utf16().count()],
                    ordinary_width: 0.0,
                    ordinary_origin5: None,
                    ordinary_raw: &[],
                    raw: &[],
                    width: None,
                },
            );
        }
    }
}

#[test]
fn rtl_inner_tab_uses_reset_s_level_and_traverses_real_l2_groups_from_the_right() {
    // UAX9 W7/I2: AB and12 have level2; L1 resets S-class TAB to base1.
    // L2 gives physical12 TAB AB. From the right, AB consumes q0..20;
    // strict T20 TAB consumes20..40;12 consumes40..60. Thus physical glyph
    // positions are1@0,2@10,A@40,B@50, not a blind reversal ofAB or12.
    for orientation in [
        LineOrientation::Vertical,
        LineOrientation::SidewaysClockwise,
        LineOrientation::SidewaysCounterclockwise,
    ] {
        qualify(
            Case {
                text: "AB\t12",
                size: TabSize::length(20.0).unwrap(),
                root_size: 20.0,
                orientation,
                inner: ParagraphBaseDirection::RightToLeft,
            },
            Expected {
                levels: &[2, 2, 1, 2, 2],
                ordinary_width: 60.0,
                ordinary_origin5: None,
                ordinary_raw: &[('A', 0.0), ('B', 10.0), ('1', 40.0), ('2', 50.0)],
                raw: &[('1', 0.0), ('2', 10.0), ('A', 40.0), ('B', 50.0)],
                width: Some(60.0),
            },
        );
    }
}

#[path = "combined_cell_inner_tabs_zero_metrics.rs"]
mod zero_metrics;

#[path = "combined_cell_inner_tabs_widths.rs"]
mod widths;

#[path = "combined_cell_inner_tabs_outer_incidence.rs"]
mod outer_incidence;

#[path = "combined_cell_range_tabs.rs"]
mod ranges;

#[path = "combined_cell_range_budgets.rs"]
mod range_budgets;

#[path = "combined_cell_range_providers.rs"]
pub(super) mod provider_ranges;
