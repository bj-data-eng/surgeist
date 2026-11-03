//! Authored word spacing is measured inside the real horizontal Cell before
//! width choice/compression (pinned Text3 §7.1 and WM4 combined-upright rules).
//! Ahem's actual negative width receipts retain normal glyphs, not fake variants.

#[path = "combined_cell_word_spacing_signed.rs"]
mod signed;

#[path = "combined_cell_word_spacing_variants.rs"]
mod variants;

#[path = "combined_cell_word_spacing_collapsed.rs"]
mod collapsed;

#[path = "combined_cell_observation_boundaries.rs"]
mod observation_boundaries;

#[path = "combined_cell_word_spacing_sideways.rs"]
mod sideways;

use super::*;
use skrifa::{GlyphId, MetadataProvider};
use surgeist_shaper::{
    Direction, Feature, FontRef as ShaperFontRef, ShaperData, ShapingStatus, Tag, UnicodeBuffer,
    WidthFeature, WidthInputOutcome, WidthObservationCoverage, script,
};

fn qualified_font() -> Vec<u8> {
    qualified_font_for("A A")
}

fn qualified_font_for(text: &str) -> Vec<u8> {
    assert!(matches!(text, "A A" | " "));
    let count = text.chars().count();
    let bytes = parent_font();
    glyph_frame_font_premises(&bytes);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    for (character, glyph) in [('A', 35), (' ', 3)]
        .into_iter()
        .chain(text.contains(' ').then_some((' ', 153)))
    {
        assert_eq!(parsed.charmap().map(character), Some(GlyphId::new(glyph)));
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
    // Observe normal and every demanded width feature on this EXACT final font.
    for request in [
        None,
        Some((*b"hwid", WidthFeature::HalfWidth)),
        Some((*b"twid", WidthFeature::ThirdWidth)),
        Some((*b"qwid", WidthFeature::QuarterWidth)),
    ] {
        let mut input = UnicodeBuffer::new();
        for (ordinal, ch) in text.chars().enumerate() {
            input.add(ch, u32::try_from(ordinal).unwrap());
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
                .map(|info| (info.glyph_id, info.cluster))
                .collect::<Vec<_>>(),
            text.chars()
                .enumerate()
                .map(|(index, ch)| {
                    let glyph = match ch {
                        'A' => 35,
                        ' ' => 3,
                        ' ' => 153,
                        _ => unreachable!(),
                    };
                    (glyph, u32::try_from(index).unwrap())
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            output
                .glyph_positions()
                .iter()
                .map(|position| (
                    position.x_advance,
                    position.y_advance,
                    position.x_offset,
                    position.y_offset
                ))
                .collect::<Vec<_>>(),
            vec![(1000, 0, 0, 0); count]
        );
        if let Some((_, feature)) = request {
            let receipt = output.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.feature(), feature);
            assert_eq!(receipt.input_len(), count);
            assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
            for ordinal in 0..count {
                assert_eq!(
                    receipt.input(ordinal).unwrap().outcome(),
                    WidthInputOutcome::NoAction
                );
            }
            assert!(receipt.input(count).is_none());
        } else {
            assert!(output.width_feature_receipt().unwrap().is_none());
        }
    }
    bytes
}

fn style(white_space: WhiteSpace, word_spacing: f64, combination: TextCombineUpright) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space,
        word_spacing,
        letter_spacing: 0.0,
        text_transform: TextTransform::none(),
        text_combine_upright: combination,
        wrap: Wrap::Word,
        ..Style::default()
    }
}

fn complete_line(prepared: &PreparedText) -> FinalizedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    prepared
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
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn native_control(
    system: &mut System,
    font: &[u8],
    word_spacing: f64,
    pens: [f64; 3],
    advance: f64,
) {
    native_control_with_orientation(
        system,
        font,
        word_spacing,
        pens,
        advance,
        LineOrientation::Horizontal,
    );
}

fn native_control_with_orientation(
    system: &mut System,
    font: &[u8],
    word_spacing: f64,
    pens: [f64; 3],
    advance: f64,
    orientation: LineOrientation,
) {
    let extent = Range::new(0, 3);
    let own = style(WhiteSpace::Preserve, word_spacing, TextCombineUpright::None);
    let mut source = Source::identified("A A", Id::from_u64(784_953), 1);
    source.span(extent, own.clone());
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: own,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
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
                anchor_order: vec![],
            },
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        Wrap::Word,
    );
    assert_eq!(analyzed.combined_cells().len(), 0);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.cell_views().len(), 0);
    let line = complete_line(&prepared);
    assert_eq!(line.cell_views().len(), 0);
    assert_eq!(line.feedback().advance, advance);
    let mut actual = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), font);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            actual.push((
                glyph.range(),
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            ));
        }
    }
    assert_eq!(
        actual,
        [
            (Range::new(0, 1), 35, pens[0], 0.0, 0.0, 10.0),
            (Range::new(1, 2), 3, pens[1], 0.0, 0.0, 10.0),
            (Range::new(2, 3), 35, pens[2], 0.0, 0.0, 10.0)
        ]
    );
}

pub(super) fn captured(
    text: &str,
    white_space: WhiteSpace,
    word_spacing: f64,
) -> (Arc<ProcessedSource>, AnalyzedSource) {
    captured_with_context(
        text,
        white_space,
        word_spacing,
        AnalysisTypography::Mixed,
        font_reference(),
        TextCombineUpright::All,
    )
}

fn captured_with_context(
    text: &str,
    white_space: WhiteSpace,
    word_spacing: f64,
    typography: AnalysisTypography,
    reference: ScopeFontReference,
    combination: TextCombineUpright,
) -> (Arc<ProcessedSource>, AnalyzedSource) {
    assert!(!text.is_empty() && text.chars().all(|ch| matches!(ch, 'A' | ' ' | '\u{a0}')));
    let length = text.encode_utf16().count();
    let extent = Range::new(0, length);
    let body = style(white_space, word_spacing, combination);
    let root = style(white_space, 0.0, TextCombineUpright::None);
    let mut source = Source::identified(text, Id::from_u64(784_954), 1);
    source.span(extent, body.clone());
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(1),
    ];
    for index in 0..length {
        if index > 0 {
            events.push(SourceProcessingEvent::Connection(index - 1));
        }
        events.push(SourceProcessingEvent::Origin {
            origin: index,
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
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..length - 1)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..length - 1)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            controls: vec![],
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
                    base_style: root,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(reference),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(body),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(reference),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
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
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        typography,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let owner = analyzed.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert!(owner.joint_flow().is_none());
    (processed, analyzed)
}

fn source_custody(
    processed: &ProcessedSource,
    analyzed: &AnalyzedSource,
    used: &[usize],
    word_spacing: f64,
) {
    source_custody_with_reference(processed, analyzed, used, word_spacing, font_reference());
}

fn source_custody_with_reference(
    processed: &ProcessedSource,
    analyzed: &AnalyzedSource,
    used: &[usize],
    word_spacing: f64,
    reference: ScopeFontReference,
) {
    assert_eq!(analyzed.combined_cells().len(), 1);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    ); // Lexical only.
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.selection_unit_count(), processed.source().text().len());
    assert_eq!(cell.used_unit_count(), used.len());
    let code_points = used
        .iter()
        .map(|&index| u32::from(processed.source().text().code_units()[index]))
        .collect::<Vec<_>>();
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), code_points);
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        code_points
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), processed.source().text().len());
    for (index, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert_eq!(piece.is_used(), used.contains(&index));
    }
    let scope = processed.captured_scope(1).unwrap();
    assert_eq!(scope.font_reference(), Some(reference));
    assert_eq!(
        scope.style().unwrap().white_space,
        analyzed.source().spans()[0].style().white_space
    );
    assert_eq!(scope.style().unwrap().word_spacing, word_spacing);
    assert_eq!(scope.style().unwrap().letter_spacing, 0.0);
    assert_eq!(scope.style().unwrap().size, 10.0);
}

fn assert_payload(
    prepared: &PreparedText,
    line: &FinalizedLine,
    processed: &ProcessedSource,
    font: &[u8],
    expected: &[(usize, u32, f64)],
    scale: f64,
    occupied: f64,
) {
    one_em(line);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual finalized Cell");
    };
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.scope_displacement().get(), 0.0);
    let mut raw = Vec::new();
    for run in line.run_views() {
        assert!(run.cell().is_some());
        assert_eq!(run.face().bytes(), font);
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        for glyph in run.glyphs() {
            raw.push((
                glyph.range(),
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            ));
        }
    }
    assert_eq!(
        raw,
        expected
            .iter()
            .map(|&(index, glyph, pen)| (Range::new(index, index + 1), glyph, pen, 0.0, 0.0, 10.0))
            .collect::<Vec<_>>()
    );
    if let Some(last) = raw.last() {
        // Public logical pens plus raw advances expose the occupied interval.
        // This is not ink, and no private interval field is fabricated.
        close(raw[0].2, 0.0);
        close(last.2 + f64::from(last.5), occupied);
    } else {
        assert_eq!(occupied, 0.0);
    }
    let layout = placed_vertical_cell(prepared, line.clone());
    assert_eq!(layout.source(), processed.source());
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let mut placed = Vec::new();
    for run in layout.glyph_runs() {
        assert_eq!(run.font().bytes(), font);
        assert_eq!(run.font_size(), 10.0);
        let frame = run.glyph_frame();
        close(frame.outline_x().x, scale);
        assert_eq!(frame.outline_x().y, 0.0);
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y(), glyph.advance()), (0.0, 0.0, 10.0));
            placed.push((
                glyph.range(),
                glyph.id(),
                frame.origin().x + scale * glyph.pen(),
                frame.origin().y,
            ));
        }
    }
    assert_eq!(placed.len(), expected.len());
    for (actual, &(index, glyph, pen)) in placed.iter().zip(expected) {
        assert_eq!((actual.0, actual.1), (Range::new(index, index + 1), glyph));
        close(actual.2, 95.0 + scale * pen);
        close(actual.3, 28.0);
    }
}

#[test]
fn preserved_inner_word_spacing_is_measured_before_normal_compression_and_retained_on_replay() {
    let font = qualified_font();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    // Same actual registered font/instance: SPACE4 splits2/2, not after-only4.
    native_control(&mut system, &font, 0.0, [0.0, 10.0, 20.0], 30.0);
    native_control(&mut system, &font, 4.0, [0.0, 12.0, 24.0], 34.0);
    let (processed, analyzed) = captured("A A", WhiteSpace::Preserve, 4.0);
    source_custody(&processed, &analyzed, &[0, 1, 2], 4.0);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .expect("authored inner WS4 must be measured before width choice, not blanket-refused");
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual prepared Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        font.as_slice()
    );
    let line = complete_line(&prepared);
    // All three width receipts are complete negative observations. Normal's
    // spaced occupied interval0..34 therefore compresses by10/34=5/17.
    let expected = [(0, 35, 0.0), (1, 3, 12.0), (2, 35, 24.0)];
    assert_payload(
        &prepared,
        &line,
        &processed,
        &font,
        &expected,
        5.0 / 17.0,
        34.0,
    );
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    source_custody(
        &processed,
        &replay.prepared_text().analyzed_source(),
        &[0, 1, 2],
        4.0,
    );
    assert_payload(
        &prepared,
        &replay,
        &processed,
        &font,
        &expected,
        5.0 / 17.0,
        34.0,
    );
}

#[test]
fn collapsed_inner_edge_spaces_keep_selection_custody_but_add_no_word_term_to_used_a() {
    let font = qualified_font();
    let (processed, analyzed) = captured(" A ", WhiteSpace::Collapse, 4.0);
    source_custody(&processed, &analyzed, &[1], 4.0);
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
        .expect("actual discarded inner edges leave A10, without a surviving SPACE word term");
    assert!(prepared.cell_views().next().unwrap().has_provider_input());
    let line = complete_line(&prepared);
    assert_payload(
        &prepared,
        &line,
        &processed,
        &font,
        &[(1, 35, 0.0)],
        1.0,
        10.0,
    );
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    source_custody(
        &processed,
        &replay.prepared_text().analyzed_source(),
        &[1],
        4.0,
    );
    assert_payload(
        &prepared,
        &replay,
        &processed,
        &font,
        &[(1, 35, 0.0)],
        1.0,
        10.0,
    );
}

#[test]
fn collapsed_whitespace_only_with_word_spacing_keeps_one_em_and_no_provider_input() {
    let font = qualified_font();
    let (processed, analyzed) = captured(" ", WhiteSpace::Collapse, 4.0);
    source_custody(&processed, &analyzed, &[], 4.0);
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
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine nonempty-source Cell");
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
    assert_payload(&prepared, &line, &processed, &font, &[], 1.0, 0.0);
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    source_custody(
        &processed,
        &replay.prepared_text().analyzed_source(),
        &[],
        4.0,
    );
    assert_payload(&prepared, &replay, &processed, &font, &[], 1.0, 0.0);
}
