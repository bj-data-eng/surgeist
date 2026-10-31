//! Native width-candidate choice over the qualified real GSUB/hmtx fixture.
//! Intended as a private child of combined_cells. No provider execution count
//! is inferred from the winning output: that remains a source-review obligation.

use super::*;

fn source_with_full_coverage(
    text: &str,
) -> (System, Arc<ProcessedSource>, AnalyzedSource, Vec<u8>) {
    assert!(!text.is_empty() && text.chars().all(|character| character == 'A'));
    let extent = Range::new(0, text.encode_utf16().count());
    let parent_style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
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
    let mut source = Source::identified(text, Id::from_u64(784_905), 75);
    // The entire real source, including every repeated A, belongs to this
    // actual Inline style and original. The single-A helper is not reused.
    source.span(extent, child_style.clone());
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
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [extent]);
    assert_eq!(
        raw.captured_paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    let domain = AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    let owner = output.captured_owner().unwrap();
    assert_eq!(owner.domain(), processed.captured_scope(0).unwrap());
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
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    ); // Lexical parent only.
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.selection_unit_count(), extent.len());
    assert_eq!(cell.used_unit_count(), extent.len());
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        vec![0x41; extent.len()]
    );
    assert_eq!(
        cell.source_pieces()
            .map(|piece| {
                assert_eq!(piece.source(), &source);
                assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
                assert!(piece.is_used());
                assert_eq!(
                    piece.origin_position(),
                    processed.captured_origin(0).unwrap()
                );
                assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
                (piece.local(), piece.aggregate())
            })
            .collect::<Vec<_>>(),
        (0..extent.len())
            .map(|index| (Range::new(index, index + 1), Range::new(index, index + 1)))
            .collect::<Vec<_>>()
    );
    let analyzed = crate::test_wrapping::bind_missing(output, Wrap::Word);
    let body = composition_font();
    glyph_frame_font_premises(&body);
    // These literal horizontal operands and real per-input GSUB actions are
    // independently qualified in test_fonts/combined_width.rs. Native vhea/
    // vmtx/BASE were added without changing any of these actual table bytes.
    for (glyph, advance) in [(35, 1000), (36, 500), (58, 400), (16, 300)] {
        assert_eq!(
            sfnt::u16_at(sfnt::table(&body, *b"hmtx"), glyph * 4),
            advance
        );
    }
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent_font()).unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 2);
    (system, processed, analyzed, body)
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-12, "{actual} != {expected}");
}

fn chosen_payload(
    text: &str,
    glyph: u32,
    raw_advance: f32,
    scale: f64,
    horizontal_offset: f64,
    physical_end: f64,
) {
    let (mut system, processed, analyzed, body) = source_with_full_coverage(text);
    let count = text.encode_utf16().count();
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
    let canonical = line
        .run_views()
        .flat_map(|run| {
            let glyphs = run
                .glyphs()
                .map(|value| (value.id(), value.advance()))
                .collect::<Vec<_>>();
            if !glyphs.is_empty() {
                assert_eq!(run.face().bytes(), body.as_slice());
            }
            glyphs
        })
        .collect::<Vec<_>>();
    assert_eq!(canonical, vec![(glyph, raw_advance); count]);
    let layout = placed_vertical_cell(&prepared, line.clone());
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.source().text(), &Text::from(text));
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let views = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let packets = layout
        .glyph_runs()
        .into_iter()
        .filter(|run| !run.glyphs().is_empty())
        .collect::<Vec<_>>();
    assert_eq!(views.len(), 1);
    assert_eq!(packets.len(), 1);
    let view = &views[0];
    let packet = &packets[0];
    assert_eq!(view.face().bytes(), body.as_slice());
    assert_eq!(packet.font().bytes(), body.as_slice());
    assert_eq!(view.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(packet.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(view.range(), Range::new(0, count));
    assert_eq!(packet.range(), Range::new(0, count));
    let frame = view.glyph_frame();
    assert_eq!(packet.glyph_frame(), frame);
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    close(frame.outline_x().x, scale);
    assert_eq!(frame.outline_x().y, 0.0);
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0)); // No vertical compression.
    close(frame.origin().x, 95.0 + horizontal_offset);
    close(frame.origin().y, 28.0); // The actual hhea8/-2 box remains vertically centered in E10.
    let raw = view
        .glyphs()
        .map(|value| {
            (
                value.id(),
                value.pen(),
                value.x(),
                value.y(),
                value.advance(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw,
        (0..count)
            .map(|index| (
                glyph,
                f64::from(u32::try_from(index).unwrap()) * f64::from(raw_advance),
                0.0,
                0.0,
                raw_advance
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        packet
            .glyphs()
            .iter()
            .map(|value| (value.id(), value.pen(), value.advance(), value.range()))
            .collect::<Vec<_>>(),
        (0..count)
            .map(|index| (
                glyph,
                f64::from(u32::try_from(index).unwrap()) * f64::from(raw_advance),
                raw_advance,
                Range::new(index, index + 1)
            ))
            .collect::<Vec<_>>()
    );
    // Preserve the horizontal advance interval under the EXECUTED glyph frame.
    // This is occupied advance, not ink; glyph outlines may exceed the square.
    let last = packet.glyphs().last().unwrap();
    close(
        frame.origin().x + frame.outline_x().x * (last.pen() + f64::from(last.advance())),
        physical_end,
    );
    let retained = prepared.analyzed_source();
    assert_eq!(
        retained
            .combined_cells()
            .next()
            .unwrap()
            .used_code_points()
            .collect::<Vec<_>>(),
        vec![0x41; count]
    );
    assert_eq!(processed.processed_text(), &Text::from(text));
}

#[test]
fn a_fitting_normal_payload_keeps_its_original_glyphs_and_unit_frame() {
    // Normal A35 has W10==E10. Width variants would replace it with36/58/16;
    // they are not the retained outcome. Output does not expose unused trials.
    chosen_payload("A", 35, 10.0, 1.0, 0.0, 105.0);
}

#[test]
fn two_as_choose_the_widest_fully_covered_candidate_that_fits() {
    // At size10: normal20, hwid10, twid8, qwid6. The widest fitting complete
    // candidate is hwid: two real glyph36 advances5, no enlargement.
    chosen_payload("AA", 36, 5.0, 1.0, 0.0, 105.0);
}

#[test]
fn three_as_keep_the_only_fitting_candidate_and_center_its_nine_unit_interval() {
    // Normal30, hwid15 and twid12 exceed E10; qwid9 fits. It stays scale1,
    // centered by (10-9)/2=.5, with three raw glyph16 advances3.
    chosen_payload("AAA", 16, 3.0, 1.0, 0.5, 104.5);
}

#[test]
fn four_as_compress_the_narrowest_covered_overwide_candidate_horizontally() {
    // Normal40, hwid20, twid16, qwid12: NONE fits. Qualified qwid is retained
    // and compressed by10/12=5/6; compressed-normal glyph35 is not permitted.
    // Raw advances3 and pens0/3/6/9 remain unchanged while the outer E stays10.
    chosen_payload("AAAA", 16, 3.0, 5.0 / 6.0, 0.0, 105.0);
}
