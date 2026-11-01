//! Genuine Inline endpoints remain exterior to accepted combined Cells.
#[cfg(feature = "text-accessibility")]
#[path = "cell_accessibility.rs"]
mod accessibility;

use super::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use std::sync::Arc;

const ORIENTATION: LineOrientation = LineOrientation::SidewaysClockwise;

fn reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    }
}

fn base(text: &str, mode: TextCombineUpright) -> ContextualSourceInput {
    let mut value = super::input(text, WhiteSpace::Preserve);
    let SourceProcessingScope::Block {
        topology,
        base_style,
        writing_axis,
        horizontal_inner_direction,
        font_reference,
        ..
    } = &mut value.scopes[0]
    else {
        unreachable!("actual root Block");
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    *font_reference = Some(reference());
    base_style.text_combine_upright = mode;
    base_style.text_transform = TextTransform::none();
    value.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
    value.scope_alignment_limits =
        Some(ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216));
    value
}

fn inline(own: Style, start: usize, end: usize) -> SourceProcessingScope {
    SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(own),
        boundaries: Some(SourceProcessingScopeBoundaries { start, end }),
        alignment_boundary: None,
        font_reference: Some(reference()),
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
    }
}

fn event(control: usize) -> SourceProcessingEvent {
    SourceProcessingEvent::Control {
        control,
        affinity: Affinity::After,
    }
}

fn analyze(value: ContextualSourceInput) -> (Arc<ProcessedSource>, AnalyzedSource) {
    let processed = Arc::new(process_contextual_source(value).unwrap());
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    (processed, analyzed)
}

fn prepare(analyzed: &AnalyzedSource) -> (System, PreparedText, Vec<u8>) {
    // CW genuinely consumes horizontal metrics; no vertical font/BASE fiction.
    // The maintained mark fixture is also used by shared_unit_endpoints tests.
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.hhea().unwrap().ascender().to_i16(), 800);
    assert_eq!(font.hhea().unwrap().descender().to_i16(), -200);
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map(0x301_u32), Some(GlyphId::new(100)));
    assert_eq!(
        crate::test_fonts::u16_at(crate::test_fonts::table(&bytes, *b"hmtx"), 35 * 4),
        1000
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(
            analyzed,
            PreparationOptions::default().with_orientation(ORIENTATION),
        )
        .unwrap();
    (system, prepared, bytes)
}

fn finish(prepared: &PreparedText, advances: &[f64]) -> FinalizedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let controls = prepared
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), advances.len());
    let allocations = controls
        .into_iter()
        .zip(advances)
        .map(|(occurrence, &advance)| ControlAllocation {
            occurrence,
            strut: None,
            boundary: Some(InlineBoundaryAllocation::contribution(advance).unwrap()),
        })
        .collect();
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
            controls: allocations,
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: ORIENTATION,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                alignment_insets: LineAlignmentInsets::ZERO,
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

#[test]
fn captured_inline_endpoints_stay_exterior_to_their_a_cell() {
    let mut value = base("A", TextCombineUpright::None);
    let body = Style {
        text_combine_upright: TextCombineUpright::All,
        ..super::style(WhiteSpace::Preserve, 10.0)
    };
    value.source.span(Range::new(0, 1), body.clone());
    value.origins = vec![super::origin(&value.source, Range::new(0, 1))];
    value.scopes.push(inline(body, 0, 1));
    value.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    value.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        event(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        event(1),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let (processed, analyzed) = analyze(value);
    assert_eq!(processed.source().text(), &Text::from("A"));
    assert_eq!(processed.processed_text(), &Text::from("A"));
    let scope = processed.captured_scope(1).unwrap();
    assert_eq!(scope.parent(), Some(processed.captured_scope(0).unwrap()));
    assert_eq!(scope.entry_control(), processed.captured_control(0));
    assert_eq!(scope.exit_control(), processed.captured_control(1));
    assert!(
        processed.captured_control(0).unwrap().event_position()
            < processed.captured_origin(0).unwrap()
    );
    assert!(
        processed.captured_origin(0).unwrap()
            < processed.captured_control(1).unwrap().event_position()
    );
    let cells = analyzed.combined_cells().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual Inline-owned A Cell");
    };
    assert_eq!(cell.scope(), scope);
    assert_eq!(
        (cell.selection_unit_count(), cell.used_unit_count()),
        (1, 1)
    );
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    let [piece] = pieces.as_slice() else {
        panic!("A keeps one original occurrence");
    };
    assert_eq!(piece.source(), processed.source());
    assert_eq!(piece.local(), Range::new(0, 1));
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(0).unwrap()
    );
    let (_system, prepared, bytes) = prepare(&analyzed);
    let controls = prepared
        .control_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    let cell = prepared.cell_views().next().unwrap();
    assert_eq!(prepared.cell_views().len(), 1);
    assert_eq!(cell.extent(), 10.0);
    assert_eq!(cell.source_cell().scope(), scope);
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        [
            PreparedParticipant::Control(controls[0].clone()),
            PreparedParticipant::Cell(cell.handle()),
            PreparedParticipant::Control(controls[1].clone()),
        ]
    );
    let line = finish(&prepared, &[2.0, 3.0]);
    assert_eq!(line.feedback().advance, 15.0); // 2 + E10 + 3, not one inner15 cell.
    assert_eq!(line.measurements().fit().advance(), 15.0);
    let square = line.cell_views().next().unwrap().square();
    assert_eq!(square.origin.x, 2.0);
    assert_eq!(square.size, Size::new(10.0, 10.0));
    let slots = line.control_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].control(), controls[0]);
    assert_eq!(slots[1].control(), controls[1]);
    assert_eq!(slots[0].strip_interval(), (0.0, 2.0));
    assert_eq!(slots[1].strip_interval(), (12.0, 15.0));
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: ORIENTATION,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    let runs = layout
        .run_views()
        .filter(|r| r.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let [run] = runs.as_slice() else {
        panic!("only A supplies a provider packet");
    };
    assert_eq!(run.face().bytes(), bytes.as_slice());
    assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(
        run.glyphs()
            .map(|g| (g.id(), g.pen(), g.advance()))
            .collect::<Vec<_>>(),
        [(35, 0.0, 10.0)]
    );
    let frame = run.glyph_frame();
    // Horizontal hhea8/-2 fits E10; CW common reference at100 puts square left98.
    // Start2 moves the square down to22, so the actual inner baseline is22+8.
    assert_eq!(frame.origin(), Point::new(98.0, 30.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
}

#[test]
fn nested_empty_inline_endpoint_pairs_do_not_split_a_cross_box_egc_or_create_cells() {
    let mut value = base("A\u{301}", TextCombineUpright::All);
    let own = Style {
        text_combine_upright: TextCombineUpright::All,
        ..super::style(WhiteSpace::Preserve, 10.0)
    };
    value.source.span(Range::new(0, 2), own.clone());
    value
        .scopes
        .extend([inline(own.clone(), 0, 3), inline(own, 1, 2)]);
    value.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|r| super::origin(&value.source, r))
        .collect();
    value.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    value.boundary_policies = vec![super::policy(ParagraphBoundaryDefinition::Connection(0))];
    value.controls = [
        ParagraphControlKind::InlineBoundaryStart,
        ParagraphControlKind::InlineBoundaryStart,
        ParagraphControlKind::InlineBoundaryEnd,
        ParagraphControlKind::InlineBoundaryEnd,
    ]
    .into_iter()
    .map(|kind| ParagraphControl { at: 1, kind })
    .collect();
    value.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::EnterScope(1),
        event(0),
        SourceProcessingEvent::EnterScope(2),
        event(1),
        event(2),
        SourceProcessingEvent::ExitScope(2),
        event(3),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let (processed, analyzed) = analyze(value);
    assert_eq!(processed.source().text(), &Text::from("A\u{301}"));
    assert_eq!(processed.processed_text(), &Text::from("A\u{301}"));
    assert_eq!(analyzed.combined_cells().len(), 0);
    assert_eq!(analyzed.inner.projection.graphemes.len(), 2); // GB9: one EGC/two edge cuts.
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|a| a.code_point)
            .collect::<Vec<_>>(),
        [0x41, 0x301]
    );
    for (scope_index, start, end) in [(1, 0, 3), (2, 1, 2)] {
        let scope = processed.captured_scope(scope_index).unwrap();
        assert_eq!(scope.entry_control(), processed.captured_control(start));
        assert_eq!(scope.exit_control(), processed.captured_control(end));
        assert_eq!(
            scope.parent(),
            Some(processed.captured_scope(scope_index - 1).unwrap())
        );
        assert!(processed.captured_origin(0).unwrap() < scope.enter_position());
        assert!(scope.exit_position() < processed.captured_origin(1).unwrap());
    }
    let (_system, prepared, _bytes) = prepare(&analyzed);
    assert_eq!(prepared.cell_views().len(), 0);
    let line = finish(&prepared, &[0.0; 4]);
    assert_eq!(line.feedback().advance, 10.0); // Real A10 plus real zero-advance attached mark.
    assert_eq!(line.cell_views().len(), 0);
    let controls = prepared
        .control_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    let slots = line.control_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 4);
    for (slot, control) in slots.iter().zip(&controls) {
        assert_eq!(slot.control(), *control);
        let (start, end) = slot.strip_interval();
        assert_eq!(start, end); // Coincidence does not erase distinct captured pairs.
    }
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert!(run.cell().is_none());
            run.glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(35, 10.0), (100, 0.0)]);
    assert_eq!(prepared.analyzed_source().source(), processed.source());
}

#[path = "cell_endpoints/box_edge.rs"]
mod box_edge;
