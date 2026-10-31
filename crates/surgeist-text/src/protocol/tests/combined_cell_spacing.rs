//! Authored outer tracking around a genuine source-nonempty/provider-empty cell.
//! The independent Text3 half-tracking oracle uses the cell as one outer unit;
//! no glyph, box, strut or width packet stands in for its empty inner payload.

use super::*;

fn tracked_empty_cell(
    letter_spacing: f64,
) -> (System, Arc<ProcessedSource>, AnalyzedSource, Vec<u8>) {
    let outside = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Collapse,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        letter_spacing: 0.0,
        word_spacing: 0.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let inside = Style {
        text_combine_upright: TextCombineUpright::All,
        letter_spacing,
        ..outside.clone()
    };
    let mut source = Source::identified("B B", Id::from_u64(784_943), 1);
    // Every original carries its actual body style. Contextual Inline.Style
    // cannot restyle the SPACE later or donate tracking to either outside B.
    source.span(Range::new(0, 1), outside.clone());
    source.span(Range::new(1, 2), inside.clone());
    source.span(Range::new(2, 3), outside.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..3)
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![
                SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: ParagraphConnectionKind::Continuous,
                },
                SourceProcessingConnection {
                    before_origin: 1,
                    after_origin: 2,
                    kind: ParagraphConnectionKind::Continuous,
                },
            ],
            controls: vec![],
            boundary_policies: (0..2)
                .map(|connection| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(connection),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: outside,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(inside),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(font_reference()),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
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
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::Connection(1),
                SourceProcessingEvent::Origin {
                    origin: 2,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("B B"));
    assert_eq!(
        processed
            .source()
            .spans()
            .iter()
            .map(|span| (
                span.range(),
                span.style().letter_spacing,
                span.style().word_spacing
            ))
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), 0.0, 0.0),
            (Range::new(1, 2), letter_spacing, 0.0),
            (Range::new(2, 3), 0.0, 0.0)
        ]
    );
    let raw = processed.paragraphs().next().unwrap();
    let domain = AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let owner = analyzed.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert!(owner.joint_flow().is_none());
    let cells = analyzed.combined_cells().collect::<Vec<_>>();
    assert_eq!(cells.len(), 1);
    let cell = &cells[0];
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    );
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.selection_unit_count(), 1);
    // Collapse is executed in the composition's actual inline-block-like inner
    // context. The authored SPACE remains source; absence is not inferred from
    // an empty numeric atom range or missing glyphs.
    assert_eq!(cell.used_unit_count(), 0);
    assert_eq!(cell.used_code_points().len(), 0);
    assert_eq!(cell.presentation_code_points().len(), 0);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    let [piece] = pieces.as_slice() else {
        panic!("one actual SPACE origin");
    };
    assert_eq!(piece.source(), processed.source());
    assert_eq!(piece.local(), Range::new(1, 2));
    assert_eq!(piece.aggregate(), Range::new(1, 2));
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(1).unwrap()
    );
    assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
    assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
    assert!(!piece.is_used());
    let inline = processed.captured_scope(1).unwrap();
    assert_eq!(inline.font_reference(), Some(font_reference()));
    assert_eq!(inline.style().unwrap().letter_spacing, letter_spacing);
    assert_eq!(inline.style().unwrap().word_spacing, 0.0);
    let body = parent_font();
    glyph_frame_font_premises(&body);
    {
        use skrifa::{
            GlyphId, MetadataProvider,
            instance::{LocationRef, Size},
        };
        let face = skrifa::FontRef::new(&body).unwrap();
        assert_eq!(face.charmap().map('B'), Some(GlyphId::new(36)));
        assert_eq!(
            face.glyph_metrics(Size::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(36)),
            Some(1000.0)
        );
    }
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    (system, processed, analyzed, body)
}

fn tracked_cell_line(
    prepared: &PreparedText,
    line: &FinalizedLine,
    processed: &ProcessedSource,
    body: &[u8],
    advance: f64,
    cell_start: f64,
    second_b: f64,
) {
    // Expectations are supplied from the Text3 zero/half-tracking rule. WM4
    // ignores INNER tracking, but makes this composition one outer glyph.
    assert_eq!(line.feedback().advance, advance);
    assert_eq!(line.measurements().fit().advance(), advance);
    assert_eq!(line.measurements().alignment().advance(), advance);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine finalized cell");
    };
    assert_eq!(cell.square().origin.x, cell_start);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.reference(), line.feedback().baseline);
    assert_eq!(cell.square().origin.y, cell.reference() - 5.0);
    assert_eq!(cell.scope_displacement().get(), 0.0);
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.source_cell().used_unit_count(), 0);
    assert!(line.run_views().all(|run| run.cell().is_none()));
    let layout = placed_vertical_cell(prepared, line.clone());
    assert_eq!(layout.source(), processed.source());
    let container = processed.captured_scope(1).unwrap();
    let metrics = prepared.scope_metrics(&container).unwrap();
    let instance = metrics.effective_instance().unwrap();
    let mut actual = Vec::new();
    for run in layout.glyph_runs() {
        assert!(run.font().face().is_same_face(&instance.face()));
        assert_eq!(run.font().bytes(), body);
        assert_eq!(run.glyph_basis(), GlyphBasis::Clockwise);
        let frame = run.glyph_frame();
        let pen = frame.pen_direction();
        for glyph in run.glyphs() {
            assert_eq!((glyph.id(), glyph.advance()), (36, 10.0));
            let x = f64::from(glyph.x()) + glyph.pen() * pen.x;
            let y = f64::from(glyph.y()) + glyph.pen() * pen.y;
            let point = Point::new(
                frame.origin().x + frame.outline_x().x * x + frame.outline_y().x * y,
                frame.origin().y + frame.outline_x().y * x + frame.outline_y().y * y,
            );
            actual.push((glyph.range(), point.y));
        }
    }
    // The actual consuming placement adds its independently supplied inline20.
    assert_eq!(
        actual,
        [
            (Range::new(0, 1), 20.0),
            (Range::new(2, 3), 20.0 + second_b)
        ]
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(1, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(95.0, 20.0 + cell_start, 10.0, 1.0)
    );
    // Native tracking assigns the following gap to this visual owner. The
    // caret ends at its flow endpoint, beyond the fixed one-em square.
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::Before))
            .unwrap()
            .rect(),
        Rect::new(95.0, 20.0 + second_b, 10.0, 1.0)
    );
    assert_eq!(layout.source().text(), &Text::from("B B"));
}

fn tracking_case(letter_spacing: f64, advance: f64, cell_start: f64, second_b: f64) {
    let (mut system, processed, analyzed, body) = tracked_empty_cell(letter_spacing);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one prepared source cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert_eq!((cell.before(), cell.after()), (5.0, 5.0));
    assert!(!cell.has_provider_input());
    assert_eq!(cell.source_cell().selection_unit_count(), 1);
    assert_eq!(cell.source_cell().used_unit_count(), 0);
    let container = processed.captured_scope(1).unwrap();
    let metrics = prepared.scope_metrics(&container).unwrap();
    assert_eq!(metrics.captured_scope(), container);
    assert_eq!(metrics.font_reference(), font_reference());
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!((metrics.over(), metrics.under()), (6.0, 6.0));
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        body.as_slice()
    );
    let line = finalize(&prepared);
    tracked_cell_line(
        &prepared, &line, &processed, &body, advance, cell_start, second_b,
    );
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    tracked_cell_line(
        &prepared, &replay, &processed, &body, advance, cell_start, second_b,
    );
}

#[test]
fn authored_tracking_spaces_a_glyph_free_combined_cell_as_one_outer_unit() {
    // Each outside B advances10. Authored2 contributes half1 on each side of
    // the middle cell: 10+1+10+1+10=32, content11..21, second B at22.
    tracking_case(2.0, 32.0, 11.0, 22.0);
}

#[test]
fn zero_tracking_keeps_the_same_glyph_free_cell_and_ordinary_glyph_positions() {
    // The same Text3 spacing term is zero: 10+0+10+0+10=30, content10..20,
    // second B at20. Placement inline20 gives B positions20/40 and carets30/40.
    tracking_case(0.0, 30.0, 10.0, 20.0);
}
