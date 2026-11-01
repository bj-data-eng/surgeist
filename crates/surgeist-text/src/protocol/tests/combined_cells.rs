//! Native preparation and one-em measurement from actual accepted source cells.
//! Font-backed normal input and source-proved empty input retain the same
//! fixed outer advance without fabricated glyph or parent-metric receipts.

use crate::*;
use crate::{baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical};
use std::sync::Arc;

fn font_reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::VerticalX,
        baseline: FontBaseline::Central,
    }
}

fn native_metric_font(horizontal: &[u8]) -> Vec<u8> {
    // Actual native vertical tables and Latn VerticalX BASE0..1200 at
    // UPEM1000. Central is600; this does not change horizontal font operands.
    let count = sfnt::u16_at(sfnt::table(horizontal, *b"maxp"), 4);
    let native = vertical::with_native(horizontal, 1200, 100, count);
    sfnt::replace_tables(
        &native,
        vec![(
            *b"BASE",
            base::base(
                0,
                None,
                Some(base::Axis {
                    tags: vec![*b"ideo", *b"idtp"],
                    scripts: vec![base::Script {
                        tag: *b"latn",
                        default_index: 0,
                        coordinates: vec![base::Coord::Design(0), base::Coord::Design(1200)],
                    }],
                }),
                None,
            ),
        )],
    )
}

fn parent_font() -> Vec<u8> {
    native_metric_font(&vertical::horizontal(800, -200, 0, 1000))
}

fn composition_font() -> Vec<u8> {
    let original = crate::shaping_test_fonts::combined_width_ahem();
    let native = native_metric_font(&original);
    // The composition's actual Inline font needs the named vertical reference.
    // Adding those real tables must preserve C's independently qualified family,
    // source mapping, GSUB actions, advances and licensed outline identities.
    for tag in [
        *b"name", *b"cmap", *b"GSUB", *b"hhea", *b"hmtx", *b"maxp", *b"glyf", *b"loca", *b"OS/2",
    ] {
        assert_eq!(sfnt::table(&native, tag), sfnt::table(&original, tag));
    }
    assert_eq!(sfnt::u16_at(sfnt::table(&native, *b"hmtx"), 35 * 4), 1000);
    native
}

fn setup(
    text: &str,
    whitespace: WhiteSpace,
) -> (System, Arc<ProcessedSource>, AnalyzedSource, Vec<u8>) {
    let parent_style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: whitespace,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let child_style = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: TextCombineUpright::All,
        ..parent_style.clone()
    };
    let mut source = Source::identified(text, Id::from_u64(784_904), 74);
    // Actual body text must carry its own computed style; Inline.Style alone
    // supplies context and cannot restyle the authored source.
    source.span(Range::new(0, 1), child_style.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, 1),
                aggregate: Range::new(0, 1),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            // Authentic scope font-reference facts require immutable capture caps,
            // even with ZERO classified alignment targets. None is not unlimited.
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: parent_style,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(child_style),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(font_reference()),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                // The parent is a genuine font context, not an extra donated strut.
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            joint_flows: vec![],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text));
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [Range::new(0, 1)]);
    let domain = AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.source(), &source);
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    assert_eq!(
        output.paragraph_strut(),
        ParagraphStrut::AllocatedContributions
    );
    let owner = output.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(0).unwrap()
    );
    assert!(owner.joint_flow().is_none());
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    // This is lexical parent metadata. The composition's metric owner is
    // the actual text-containing Inline1, with its own effective C-family font.
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    );
    assert_eq!(
        processed.captured_scope(1).unwrap().font_reference(),
        Some(font_reference())
    );
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.selection_unit_count(), 1);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].source(), &source);
    assert_eq!(pieces[0].local(), Range::new(0, 1));
    assert_eq!(pieces[0].aggregate(), Range::new(0, 1));
    assert_eq!(
        pieces[0].origin_position(),
        processed.captured_origin(0).unwrap()
    );
    assert_eq!(
        pieces[0].paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(pieces[0].kind(), ParagraphOriginKind::Authored);
    let analyzed = crate::test_wrapping::bind_missing(output, Wrap::Word);
    let body = composition_font();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent_font()).unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 2);
    (system, processed, analyzed, body)
}

fn finalize(prepared: &PreparedText) -> FinalizedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let selected = prepared
        .select(LineSelectionRequest {
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
                orientation: LineOrientation::Vertical,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap();
    selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn one_em(line: &FinalizedLine) {
    assert!(line.has_in_flow_content());
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.measurements().fit().advance(), 10.0);
    assert_eq!(line.measurements().alignment().advance(), 10.0);
    assert_eq!(
        line.measurements().min_content_contribution().advance(),
        10.0
    );
    assert_eq!(
        line.measurements().max_content_contribution().advance(),
        10.0
    );
    assert_eq!(line.metric_bounds().size.width, 10.0);
}

#[test]
fn normal_fitting_combined_a_prepares_as_one_em_without_width_substitution() {
    let (mut system, processed, analyzed, body) = setup("A", WhiteSpace::Preserve);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.used_unit_count(), 1);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    assert_eq!(cell.presentation_code_points().collect::<Vec<_>>(), [0x41]);
    assert!(cell.source_pieces().next().unwrap().is_used());
    assert_eq!(
        processed
            .plaintext_items(Affinity::After)
            .flat_map(|item| match item {
                PlaintextItem::Text(text) => text.code_units().to_vec(),
                PlaintextItem::InlineBox(_) => vec![],
            })
            .collect::<Vec<_>>(),
        [0x41]
    );
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), processed.source());
    let line = finalize(&prepared);
    one_em(&line);
    // Qualified font has UPEM1000/A35 advance1000. Size10 fits E10 normally:
    // no hwid/twid/qwid substitution (glyph36/58/16) or enlargement is needed.
    assert_eq!(
        line.run_views()
            .flat_map(|run| {
                let glyphs = run
                    .glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>();
                if !glyphs.is_empty() {
                    assert_eq!(run.face().bytes(), body.as_slice());
                }
                glyphs
            })
            .collect::<Vec<_>>(),
        [(35, 10.0)]
    );
    assert_eq!(line.prepared_text(), prepared);
    assert_eq!(processed.source().text(), &Text::from("A"));
}

#[test]
fn source_empty_combined_payload_still_measures_one_em_without_a_body_glyph() {
    let (mut system, processed, analyzed, _) = setup(" ", WhiteSpace::Collapse);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.used_unit_count(), 0);
    assert_eq!(cell.used_code_points().len(), 0);
    assert_eq!(cell.presentation_code_points().len(), 0);
    assert!(!cell.source_pieces().next().unwrap().is_used());
    assert_eq!(analyzed.inner.projection.atoms.len(), 0);
    assert_eq!(processed.source().text(), &Text::from(" "));
    assert_eq!(processed.processed_text(), &Text::from(" "));
    assert_eq!(
        processed
            .plaintext_items(Affinity::After)
            .flat_map(|item| match item {
                PlaintextItem::Text(text) => text.code_units().to_vec(),
                PlaintextItem::InlineBox(_) => vec![],
            })
            .collect::<Vec<_>>(),
        []
    );
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), processed.source());
    let line = finalize(&prepared);
    one_em(&line);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().len())
            .sum::<usize>(),
        0
    );
    assert_eq!(line.prepared_text(), prepared);
    assert_eq!(
        analyzed
            .combined_cells()
            .next()
            .unwrap()
            .source_pieces()
            .next()
            .unwrap()
            .source(),
        processed.source()
    );
}

fn glyph_frame_font_premises(bytes: &[u8]) {
    use skrifa::{
        GlyphId, MetadataProvider,
        instance::{LocationRef, Size},
        raw::TableProvider,
    };
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.hhea().unwrap().ascender().to_i16(), 800);
    assert_eq!(font.hhea().unwrap().descender().to_i16(), -200);
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    let glyphs = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(glyphs.advance_width(GlyphId::new(35)), Some(1000.0));
    let bounds = glyphs.bounds(GlyphId::new(35)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
    let table = font.base().unwrap();
    let axis = table.vert_axis().unwrap().unwrap();
    assert_eq!(
        axis.base_tag_list()
            .unwrap()
            .unwrap()
            .baseline_tags()
            .iter()
            .map(|tag| tag.get().to_be_bytes())
            .collect::<Vec<_>>(),
        [*b"ideo", *b"idtp"]
    );
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(scripts.base_script_records().len(), 1);
    let script = &scripts.base_script_records()[0];
    assert_eq!(script.base_script_tag().to_be_bytes(), *b"latn");
    let values = script
        .base_script(scripts.offset_data())
        .unwrap()
        .base_values()
        .unwrap()
        .unwrap();
    assert_eq!(
        values
            .base_coords()
            .iter()
            .map(|coordinate| coordinate.unwrap().coordinate())
            .collect::<Vec<_>>(),
        [0, 1200]
    );
}

fn placed_vertical_cell(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
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
    .unwrap()
}

#[test]
fn a_native_vertical_cell_exposes_its_horizontal_glyph_frame_in_borrowed_and_owned_runs() {
    let (mut system, processed, analyzed, body) = setup("A", WhiteSpace::Preserve);
    glyph_frame_font_premises(&body);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let line = finalize(&prepared);
    one_em(&line);
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    assert_eq!(layout.lines().len(), 1);
    // Actual Inline's BASE edges are0/12 with Central6 at size10: O6/U-6
    // about the supplied common baseline100. E10 therefore spans x95..105;
    // inline20 gives y20..30. No lexical-root or aggregate-line metrics enter.
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let borrowed = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let owned = layout
        .glyph_runs()
        .into_iter()
        .filter(|run| !run.glyphs().is_empty())
        .collect::<Vec<_>>();
    assert_eq!(borrowed.len(), 1);
    assert_eq!(owned.len(), 1);
    let view = &borrowed[0];
    let packet = &owned[0];
    assert_eq!(view.orientation(), LineOrientation::Vertical);
    assert_eq!(packet.orientation(), LineOrientation::Vertical);
    assert_eq!(view.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(packet.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(view.face().bytes(), body.as_slice());
    assert_eq!(packet.font().bytes(), body.as_slice());
    assert_eq!(view.range(), Range::new(0, 1));
    assert_eq!(packet.range(), Range::new(0, 1));
    assert_eq!(packet.glyphs()[0].range(), Range::new(0, 1));
    assert_eq!(
        view.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 10.0)]
    );
    assert_eq!(
        packet
            .glyphs()
            .iter()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 10.0)]
    );
    let frame = view.glyph_frame();
    assert_eq!(packet.glyph_frame(), frame);
    // Normal A occupies horizontal[0,10], so scale1 and centering offset0.
    // Its verified hhea8/-2 metric box already has heightE10: baseline8.
    // The font-design origin is thus square-left95, square-top20+8=28.
    assert_eq!(frame.origin(), Point::new(95.0, 28.0));
    assert_eq!(view.origin(), Point::new(95.0, 28.0));
    assert_eq!(packet.origin(), Point::new(95.0, 28.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    // Qualified A ink is0..10 x -8..2 in converted layout coordinates.
    // Use the observed executable axes with the independent literal corners.
    let top = Point::new(
        frame.origin().x - 8.0 * frame.outline_y().x,
        frame.origin().y - 8.0 * frame.outline_y().y,
    );
    let bottom = Point::new(
        frame.origin().x + 10.0 * frame.outline_x().x + 2.0 * frame.outline_y().x,
        frame.origin().y + 10.0 * frame.outline_x().y + 2.0 * frame.outline_y().y,
    );
    assert_eq!(top, Point::new(95.0, 20.0));
    assert_eq!(bottom, Point::new(105.0, 30.0));
    let prepared_source = prepared.analyzed_source();
    let cell = prepared_source.combined_cells().next().unwrap();
    let original = cell.source_pieces().next().unwrap();
    assert_eq!(original.source(), processed.source());
    assert_eq!(original.aggregate(), Range::new(0, 1));
    assert_eq!(
        original.origin_position(),
        processed.captured_origin(0).unwrap()
    );
}

#[test]
fn an_accepted_source_empty_cell_places_a_real_one_em_square_without_a_glyph_run() {
    let (mut system, processed, analyzed, body) = setup(" ", WhiteSpace::Collapse);
    glyph_frame_font_premises(&body);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.selection_unit_count(), 1);
    assert_eq!(cell.used_unit_count(), 0);
    assert_eq!(cell.presentation_code_points().len(), 0);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let line = finalize(&prepared);
    one_em(&line);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().len())
            .sum::<usize>(),
        0
    );
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.source().text(), &Text::from(" "));
    assert!(layout.source().boxes().is_empty());
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.nonformatted_lines().count(), 0);
    // Same actual composition font and anchor as the A case. Empty provider
    // input has no fake run/frame/glyph, but keeps the E10 metric square.
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    assert_eq!(
        layout
            .run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    let prepared_source = prepared.analyzed_source();
    let original = prepared_source
        .combined_cells()
        .next()
        .unwrap()
        .source_pieces()
        .next()
        .unwrap();
    assert_eq!(original.source(), processed.source());
    assert_eq!(original.local(), Range::new(0, 1));
    assert_eq!(original.aggregate(), Range::new(0, 1));
    assert!(!original.is_used());
    assert_eq!(
        original.origin_position(),
        processed.captured_origin(0).unwrap()
    );
}

fn cell_interaction_contract(text: &str, whitespace: WhiteSpace, plaintext: &str) {
    let (mut system, processed, analyzed, body) = setup(text, whitespace);
    glyph_frame_font_premises(&body);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let layout = placed_vertical_cell(&prepared, finalize(&prepared));
    let start = Cursor::new(0, Affinity::After);
    let end = Cursor::new(1, Affinity::Before);
    let positions = layout.positions().collect::<Vec<_>>();
    // One indivisible outer cell has two real selected-event edges. Neither a
    // provider glyph nor an executed removal introduces an inner outer caret.
    assert_eq!(
        positions
            .iter()
            .map(PlacedPosition::source_cursor)
            .collect::<Vec<_>>(),
        [start, end]
    );
    let before = Rect::new(95.0, 20.0, 10.0, 1.0);
    let after = Rect::new(95.0, 30.0, 10.0, 1.0);
    // Genuine Inline1 font BASE gives O6/U-6 and E10. At physical anchor
    // inline20/baseline100 the square is95,20,10,10; carets span its cross axis.
    for (position, source, expected) in
        [(&positions[0], start, before), (&positions[1], end, after)]
    {
        assert_eq!(layout.position_geometry(position).unwrap().rect(), expected);
        assert_eq!(layout.cursor(source).unwrap().rect(), expected);
    }
    for movement in [
        Movement::NextCluster,
        Movement::NextWord,
        Movement::LineEnd,
        Movement::DocumentEnd,
    ] {
        let moved = layout.move_position(&positions[0], movement).unwrap();
        assert_eq!(moved, positions[1]);
        assert_eq!(layout.move_cursor(start, movement).unwrap(), end);
        assert_eq!(layout.position_geometry(&moved).unwrap().rect(), after);
    }
    for movement in [
        Movement::PreviousCluster,
        Movement::PreviousWord,
        Movement::LineStart,
        Movement::DocumentStart,
    ] {
        let moved = layout.move_position(&positions[1], movement).unwrap();
        assert_eq!(moved, positions[0]);
        assert_eq!(layout.move_cursor(end, movement).unwrap(), start);
        assert_eq!(layout.position_geometry(&moved).unwrap().rect(), before);
    }
    for (point, expected) in [
        (Point::new(100.0, 21.0), &positions[0]),
        (Point::new(100.0, 25.0), &positions[0]),
        (Point::new(100.0, 29.0), &positions[1]),
    ] {
        let Hit::Text(hit) = layout.hit(point) else {
            panic!("the real cell square is text hit geometry");
        };
        assert_eq!(&hit, expected);
    }
    let square = SelectionGeometry::new(vec![SelectionRect::new(
        Rect::new(95.0, 20.0, 10.0, 10.0),
        0,
    )]);
    assert_eq!(
        layout.selection(Selection::new(start, end)).unwrap(),
        square
    );
    assert_eq!(
        layout.selection(Selection::new(end, start)).unwrap(),
        square
    );
    for (anchor, focus) in [
        (&positions[0], &positions[1]),
        (&positions[1], &positions[0]),
    ] {
        let selected = layout.position_selection(anchor, focus).unwrap();
        assert!(!selected.is_collapsed());
        assert_eq!(selected.geometry(), &square);
        assert_eq!(selected.source_text(), &Text::from(text));
        assert_eq!(selected.plaintext(), &Text::from(plaintext));
        let [piece] = selected.source_pieces() else {
            panic!("one actual captured source occurrence");
        };
        assert_eq!(piece.local_range(), Range::new(0, 1));
        assert_eq!(piece.aggregate_range(), Range::new(0, 1));
        assert_eq!(piece.code_units(), processed.source().text().code_units());
        assert_eq!(
            piece.origin(),
            &prepared.origin_views().next().unwrap().handle()
        );
        assert_eq!(
            piece.origin().view().capture_position(),
            Some(processed.captured_origin(0).unwrap())
        );
    }
    assert!(
        layout
            .position_selection(&positions[0], &positions[0])
            .unwrap()
            .is_collapsed()
    );
    assert!(
        layout
            .selection(Selection::collapsed(start))
            .unwrap()
            .rects()
            .is_empty()
    );
    assert_eq!(layout.source(), processed.source());
}

#[test]
fn combined_a_interaction_uses_actual_cell_edges_square_and_original_occurrence() {
    cell_interaction_contract("A", WhiteSpace::Preserve, "A");
}

#[test]
fn collapsed_space_cell_interaction_keeps_the_original_source_without_a_provider_glyph() {
    cell_interaction_contract(" ", WhiteSpace::Collapse, "");
}

#[path = "combined_cell_resources.rs"]
mod resources;

#[path = "combined_cell_widths.rs"]
mod widths;

#[path = "combined_cell_containers.rs"]
mod containers;

#[path = "combined_cell_script_context.rs"]
mod script_context;

#[path = "combined_cell_spacing.rs"]
mod spacing;

#[path = "combined_cell_inner_controls.rs"]
mod inner_controls;

#[path = "combined_cell_inner_tabs.rs"]
mod inner_tabs;

#[path = "combined_cells/inline_box_family.rs"]
mod inline_box_family;
