//! Actual variable ordinary text and two differently reduced cells share one
//! preparation/selection. Literal Source ranges and font tables own the oracle.
use super::*;

fn add_variable(static_font: &[u8], variable: &[u8]) -> Vec<u8> {
    let bytes = sfnt::replace_tables(
        static_font,
        [*b"fvar", *b"gvar"]
            .into_iter()
            .map(|tag| (tag, sfnt::table(variable, tag).to_vec()))
            .collect(),
    );
    for tag in [
        *b"name", *b"cmap", *b"GSUB", *b"hmtx", *b"vhea", *b"vmtx", *b"BASE", *b"glyf", *b"loca",
        *b"maxp",
    ] {
        // Ordinary Ahem has no GSUB; qualify that genuine absence separately.
        if tag == *b"GSUB" && !has_table(static_font, tag) {
            continue;
        }
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(static_font, tag));
    }
    for tag in [*b"fvar", *b"gvar"] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(variable, tag));
    }
    bytes
}

fn has_table(bytes: &[u8], tag: [u8; 4]) -> bool {
    (0..usize::from(sfnt::u16_at(bytes, 4)))
        .any(|index| bytes[12 + index * 16..16 + index * 16] == tag)
}

fn mixed_width_source() -> (
    System,
    Arc<ProcessedSource>,
    AnalyzedSource,
    FontData,
    FontData,
) {
    use skrifa::{
        GlyphId, MetadataProvider,
        instance::{LocationRef, Size},
        raw::TableProvider,
    };
    let variable = crate::test_fonts::variable_ahem();
    let fvar = sfnt::table(&variable, *b"fvar");
    assert_eq!(&fvar[16..20], b"wght");
    assert_eq!(&fvar[20..32], &[0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0]); // min/default0,max1.
    // Native tables use real static outlines first; append actual fvar/gvar
    // afterward, preserving both families and every native BASE operand.
    let static_parent = parent_font();
    let static_body = composition_font();
    glyph_frame_font_premises(&static_body);
    let parent = add_variable(&static_parent, &variable);
    let body = add_variable(&static_body, &variable);
    let face = skrifa::FontRef::new(&parent).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.charmap().map('X'), Some(GlyphId::new(58)));
    assert_eq!(
        face.glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(58)),
        Some(1000.0)
    );
    assert_eq!(sfnt::u16_at(sfnt::table(&parent, *b"hmtx"), 58 * 4), 1000);
    for (glyph, advance) in [(35, 1000), (36, 500), (58, 400), (16, 300)] {
        assert_eq!(
            sfnt::u16_at(sfnt::table(&body, *b"hmtx"), glyph * 4),
            advance
        );
    }
    // variable_ahem's authored X tuple moves right outline points by100; all
    // phantom deltas are0. At wght1, X ink grows to1100 but hmtx advance stays
    // 1000. Width-cell choice uses measured advance, never that ink width.
    let outside = Style {
        font: Font::new().family("Ahem").variation("\"wght\" 1"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        text_combine_upright: TextCombineUpright::None,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let inside = Style {
        font: Font::new()
            .family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY)
            .variation("\"wght\" 1"),
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: TextCombineUpright::All,
        ..outside.clone()
    };
    let mut source = Source::identified("XAAXAAAAX", Id::from_u64(784_941), 1);
    source.span(Range::new(1, 3), inside.clone());
    source.span(Range::new(4, 8), inside.clone());
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for index in 0..9 {
        if index > 0 {
            events.push(SourceProcessingEvent::Connection(index - 1));
        }
        if index == 1 {
            events.push(SourceProcessingEvent::EnterScope(1));
        }
        if index == 4 {
            events.push(SourceProcessingEvent::EnterScope(2));
        }
        events.push(SourceProcessingEvent::Origin {
            origin: index,
            affinity: Affinity::After,
        });
        if index == 2 {
            events.push(SourceProcessingEvent::ExitScope(1));
        }
        if index == 7 {
            events.push(SourceProcessingEvent::ExitScope(2));
        }
    }
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let inline = || SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(inside.clone()),
        boundaries: None,
        alignment_boundary: None,
        font_reference: Some(font_reference()),
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
    };
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..9)
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..8)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            // The genuine incident scopes and each real common ancestor all use
            // Word; each actual connection retains its own admitted policy row.
            boundary_policies: (0..8)
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
                    base_style: outside,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                inline(),
                inline(),
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
    assert_eq!(processed.processed_text(), &Text::from("XAAXAAAAX"));
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(
        raw.source_pieces().collect::<Vec<_>>(),
        (0..9)
            .map(|index| Range::new(index, index + 1))
            .collect::<Vec<_>>()
    );
    let domain = AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    source_cells(&analyzed, &processed);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent.clone()).unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 2);
    let parent = system
        .registered_font_faces()
        .find(|face| face.bytes() == parent.as_slice())
        .unwrap()
        .to_owned();
    let body = system
        .registered_font_faces()
        .find(|face| face.bytes() == body.as_slice())
        .unwrap()
        .to_owned();
    assert!(!parent.face().is_same_face(&body.face()));
    (system, processed, analyzed, parent, body)
}

fn source_cells(analyzed: &AnalyzedSource, processed: &Arc<ProcessedSource>) {
    assert_eq!(analyzed.source(), processed.source());
    let owner = analyzed.captured_owner().unwrap();
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
    let cells = analyzed.combined_cells().collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    for (cell, (scope, start, end)) in cells.iter().zip([(1, 1, 3), (2, 4, 8)]) {
        assert_eq!(cell.scope(), processed.captured_scope(scope).unwrap());
        assert_eq!(
            cell.parent_scope(),
            Some(processed.captured_scope(0).unwrap())
        );
        assert_eq!(cell.selection_unit_count(), end - start);
        assert_eq!(cell.used_unit_count(), end - start);
        assert_eq!(
            cell.used_code_points().collect::<Vec<_>>(),
            vec![0x41; end - start]
        );
        assert_eq!(
            cell.presentation_code_points().collect::<Vec<_>>(),
            vec![0x41; end - start]
        );
        let pieces = cell.source_pieces().collect::<Vec<_>>();
        assert_eq!(pieces.len(), end - start);
        for (piece, index) in pieces.iter().zip(start..end) {
            assert_eq!(piece.source(), processed.source());
            assert_eq!(piece.local(), Range::new(index, index + 1));
            assert_eq!(piece.aggregate(), Range::new(index, index + 1));
            assert_eq!(
                piece.origin_position(),
                processed.captured_origin(index).unwrap()
            );
            assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
            assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
            assert!(piece.is_used());
        }
    }
}

fn actual_coordinates(
    shaped: &crate::engine::layout::data::ShapedData,
    parent: &FontData,
    body: &FontData,
) {
    // Owning protocol qualification reads the actual canonical buffers. It does
    // not construct a new instance, normalize authored values, or guess run count.
    assert!(!shaped.runs.is_empty());
    for run in shaped.runs.iter().chain(shaped.empty_run.iter()) {
        assert!(
            run.font.face().is_same_face(&parent.face())
                || run.font.face().is_same_face(&body.face())
        );
        assert_eq!(run.font.catalogue_identity(), parent.catalogue_identity());
        assert_eq!(run.font_size, 10.0);
        assert_eq!(run.coords, [16384]);
    }
}

fn expected_glyphs() -> Vec<(Range, u32, f64, f32, f32, f32)> {
    vec![
        (Range::new(0, 1), 58, 0.0, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 36, 0.0, 0.0, 0.0, 5.0),
        (Range::new(2, 3), 36, 5.0, 0.0, 0.0, 5.0),
        (Range::new(3, 4), 58, 0.0, 0.0, 0.0, 10.0),
        (Range::new(4, 5), 16, 0.0, 0.0, 0.0, 3.0),
        (Range::new(5, 6), 16, 3.0, 0.0, 0.0, 3.0),
        (Range::new(6, 7), 16, 6.0, 0.0, 0.0, 3.0),
        (Range::new(7, 8), 16, 9.0, 0.0, 0.0, 3.0),
        (Range::new(8, 9), 58, 0.0, 0.0, 0.0, 10.0),
    ]
}

fn finalized(
    line: &FinalizedLine,
    processed: &Arc<ProcessedSource>,
    parent: &FontData,
    body: &FontData,
) {
    // 3 ordinary advances10 +2 genuine outer cells10, independent of inner
    // raw qwid advance12 and of variable X's overhanging outline width11.
    assert_eq!(line.feedback().advance, 50.0);
    let cells = line.cell_views().collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    for (cell, (scope, inline)) in cells.iter().zip([(1, 10.0), (2, 30.0)]) {
        assert_eq!(
            cell.source_cell().scope(),
            processed.captured_scope(scope).unwrap()
        );
        assert_eq!(cell.square().origin.x, inline);
        assert_eq!(cell.square().size, Size::new(10.0, 10.0));
        assert_eq!(
            cell.scope_displacement(),
            BaselineShift::try_new(0.0).unwrap()
        );
    }
    let rows = line
        .run_views()
        .flat_map(|run| {
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.normalized_coords().collect::<Vec<_>>(), [16384]);
            if let Some(cell) = run.cell() {
                assert!(run.face().is_same_face(&body.face()));
                let owner = cell.source_cell().scope();
                assert!(
                    owner == processed.captured_scope(1).unwrap()
                        || owner == processed.captured_scope(2).unwrap()
                );
            } else {
                assert!(run.face().is_same_face(&parent.face()));
            }
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.pen(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(rows, expected_glyphs());
}

fn placed(layout: &Layout, processed: &Arc<ProcessedSource>, parent: &FontData, body: &FontData) {
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.source().text(), &Text::from("XAAXAAAAX"));
    source_cells(&layout.analyzed_source(), processed);
    assert_eq!(layout.lines().len(), 1);
    let views = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let packets = layout
        .glyph_runs()
        .into_iter()
        .filter(|run| !run.glyphs().is_empty())
        .collect::<Vec<_>>();
    assert_eq!(views.len(), packets.len());
    let mut rows = Vec::new();
    for (view, packet) in views.iter().zip(&packets) {
        assert_eq!(
            view.effective_instance()
                .unwrap()
                .normalized_coords()
                .collect::<Vec<_>>(),
            [16384]
        );
        assert_eq!(view.range(), packet.range());
        assert_eq!(view.face().bytes(), packet.font().bytes());
        assert_eq!(view.glyph_frame(), packet.glyph_frame());
        assert_eq!(
            view.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>(),
            packet
                .glyphs()
                .iter()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>()
        );
        let range = packet.range();
        if range == Range::new(1, 3) || range == Range::new(4, 8) {
            assert_eq!(packet.font(), body);
            assert_eq!(packet.glyph_basis(), GlyphBasis::Unrotated);
            let frame = packet.glyph_frame();
            assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
            assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
            let (scale, y) = if range == Range::new(1, 3) {
                (1.0, 38.0)
            } else {
                (5.0 / 6.0, 58.0)
            };
            close(frame.outline_x().x, scale);
            assert_eq!(frame.outline_x().y, 0.0);
            close(frame.origin().x, 95.0);
            close(frame.origin().y, y);
            let last = packet.glyphs().last().unwrap();
            close(
                frame.origin().x + scale * (last.pen() + f64::from(last.advance())),
                105.0,
            );
        } else {
            assert_eq!(packet.font(), parent);
            assert!(matches!((range.start, range.end), (0, 1) | (3, 4) | (8, 9)));
        }
        rows.extend(
            packet
                .glyphs()
                .iter()
                .map(|g| (g.range(), g.id(), g.pen(), g.x(), g.y(), g.advance())),
        );
    }
    assert_eq!(rows, expected_glyphs());
}

#[test]
fn mixed_variable_ordinary_rows_and_two_width_cells_keep_canonical_payloads_through_replay() {
    let (mut system, processed, analyzed, parent, body) = mixed_width_source();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    actual_coordinates(&prepared.inner.shaped, &parent, &body);
    assert_eq!(prepared.cell_views().len(), 2);
    for scope in [1, 2] {
        let metrics = prepared
            .scope_metrics(&processed.captured_scope(scope).unwrap())
            .unwrap();
        assert_eq!(metrics.font_size(), 10.0);
        let instance = metrics.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&body.face()));
        assert_eq!(instance.normalized_coords().collect::<Vec<_>>(), [16384]);
    }
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let selected = prepared
        .select(LineSelectionRequest {
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
    actual_coordinates(&selected.inner.basis.shaped, &parent, &body);
    drop(system); // Selection/placement/replay retain real faces and coordinates.
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    finalized(&line, &processed, &parent, &body);
    let layout = placed_vertical_cell(&prepared, line.clone());
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    placed(&layout, &processed, &parent, &body);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.selected_line(), selected);
    finalized(&replay, &processed, &parent, &body);
    let replay_layout = placed_vertical_cell(&prepared, replay);
    assert_eq!(
        replay_layout.analyzed_source().identity(),
        analyzed.identity()
    );
    placed(&replay_layout, &processed, &parent, &body);
    // Recheck the earlier immutable results against the literal cohort too.
    finalized(&line, &processed, &parent, &body);
    placed(&layout, &processed, &parent, &body);
}
