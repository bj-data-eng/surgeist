//! One genuine original style owns both inner and ordinary CSS TAB occurrences.
use super::*;

fn mixed_capture() -> Captured {
    let body = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        text_combine_upright: TextCombineUpright::All,
        text_transform: TextTransform::none(),
        tab_size: TabSize::spaces(2.0).unwrap(),
        letter_spacing: 0.0,
        word_spacing: 0.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let root_style = Style {
        size: 20.0,
        line_height: LineHeight::Absolute(20.0),
        text_combine_upright: TextCombineUpright::None,
        ..body.clone()
    };
    let mut source = Source::identified("A\tBA\tB", Id::from_u64(784_992_002), 1);
    source.span(Range::new(0, 6), body.clone());
    let inline = || SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(body.clone()),
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
            origins: (0..6)
                .map(|at| ParagraphOrigin {
                    text_node: None,
                    // All genuine originals refer to the one authored source, including
                    // its single full-range style. No context-only restyling occurs.
                    source: source.clone(),
                    local: Range::new(at, at + 1),
                    aggregate: Range::new(at, at + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..5)
                .map(|before| SourceProcessingConnection {
                    before_origin: before,
                    after_origin: before + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..5)
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
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: root_style,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                inline(),
                inline(),
                inline(),
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(1),
                SourceProcessingEvent::EnterScope(2),
                SourceProcessingEvent::Origin {
                    origin: 2,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(2),
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::Connection(2),
                SourceProcessingEvent::EnterScope(3),
                SourceProcessingEvent::Origin {
                    origin: 3,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(3),
                SourceProcessingEvent::Origin {
                    origin: 4,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(4),
                SourceProcessingEvent::Origin {
                    origin: 5,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(3),
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    // The first All10 box genuinely contains a nested All10 box. The existing
    // WM4 box lookaround rejects both partial sequences, so its A/TAB/B remain
    // ordinary. The sibling All10 has no All ancestor across that box boundary
    // (Root is None), so precisely its A/TAB/B form one accepted Cell.
    let root = processed.captured_scope(0).unwrap();
    let outer = processed.captured_scope(1).unwrap();
    let nested = processed.captured_scope(2).unwrap();
    let cell_scope = processed.captured_scope(3).unwrap();
    assert_eq!(outer.parent(), Some(root.clone()));
    assert_eq!(nested.parent(), Some(outer));
    assert_eq!(cell_scope.parent(), Some(root));
    for scope in [1, 2, 3] {
        assert_eq!(
            processed.captured_scope(scope).unwrap().style(),
            Some(&body)
        );
    }
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::Mixed,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(analyzed.combined_cells().len(), 1);
    assert_eq!(cell.scope(), cell_scope);
    assert_eq!(cell.selection_unit_count(), 3);
    assert_eq!(cell.used_unit_count(), 3);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [65, 9, 66]);
    assert_eq!(
        cell.inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        [0, 0, 0]
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 3);
    for (piece, at) in pieces.iter().zip(3..6) {
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(at).unwrap()
        );
        assert_eq!(piece.local(), Range::new(at, at + 1));
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
        assert!(piece.is_used());
    }
    let tabs = tab_keys(&analyzed);
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].1, tabs[1].1);
    assert_eq!(analyzed.inner.styles[tabs[0].1].style.authored(), &body);
    let bindings = &analyzed.inner.combination.as_ref().unwrap().bindings;
    assert_eq!(bindings.len(), 1);
    assert!(!bindings[0].atoms.contains(&tabs[0].0));
    assert!(bindings[0].atoms.contains(&tabs[1].0));
    Captured {
        processed,
        analyzed,
    }
}

fn tab_keys(analyzed: &AnalyzedSource) -> Vec<(usize, usize)> {
    analyzed
        .inner
        .projection
        .atoms
        .iter()
        .enumerate()
        .filter(|(_, atom)| atom.code_point == 9)
        // This is the canonical actual style key consumed by the outer TAB
        // preparation scan, not equality of independently cloned styles.
        .map(|(atom, _)| (atom, analyzed.inner.analysis.infos[atom].1))
        .collect()
}

fn verify_mixed(
    prepared: &PreparedText,
    line: &FinalizedLine,
    bytes: &[u8],
    outer_b: f64,
    cell_start: f64,
    advance: f64,
) {
    assert_eq!(line.feedback().advance, advance);
    assert_eq!(line.cell_views().len(), 1);
    let cell = line.cell_views().next().unwrap();
    assert_eq!(cell.square().origin.x, cell_start);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    let mut ordinary = vec![];
    let mut inner = vec![];
    for run in line.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.font_size(), 10.0);
        let target = if run.cell().is_some() {
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            &mut inner
        } else {
            assert_eq!(run.glyph_basis(), GlyphBasis::Clockwise);
            &mut ordinary
        };
        for glyph in run.glyphs() {
            assert_eq!((glyph.advance(), glyph.x(), glyph.y()), (10.0, 0.0, 0.0));
            target.push((glyph.id(), run.origin().x + glyph.pen()));
        }
    }
    ordinary.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    inner.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    assert_eq!(ordinary, [(35, 0.0), (36, outer_b)]);
    assert_eq!(inner, [(35, 0.0), (36, 20.0)]);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    let mut physical_inner = vec![];
    for run in layout
        .run_views()
        .filter(|run| run.glyph_basis() == GlyphBasis::Unrotated)
    {
        if run.glyphs().next().is_none() {
            continue;
        }
        let frame = run.glyph_frame();
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        close(frame.outline_x().x, 1.0 / 3.0);
        assert_eq!(frame.outline_x().y, 0.0);
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        close(frame.origin().y, 20.0 + cell_start + 8.0);
        for glyph in run.glyphs() {
            physical_inner.push((
                glyph.id(),
                frame.origin().x + frame.outline_x().x * glyph.pen(),
            ));
        }
    }
    physical_inner.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    assert_eq!(
        physical_inner.iter().map(|&(id, _)| id).collect::<Vec<_>>(),
        [35, 36]
    );
    close(physical_inner[0].1, 95.0);
    close(physical_inner[1].1, 95.0 + 20.0 / 3.0);
}

#[test]
fn one_style_keeps_outer_tab_origin_demand_and_inner_container_zero_grid_distinct() {
    let bytes = qualified_font();
    let capture = mixed_capture();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(&capture.analyzed, options(LineOrientation::Vertical))
        .unwrap();
    let tabs = tab_keys(&prepared.analyzed_source());
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].1, tabs[1].1);
    // Real prepared style resolution preserves the canonical index consumed by
    // both occurrences; excluding an entire style would lose the ordinary TAB.
    assert_eq!(prepared.inner.styles[tabs[0].1].style.font_size, 10.0);
    assert_eq!(prepared.cell_views().len(), 1);
    container_premises_for_scope(&prepared, &capture, &bytes);
    let without_origin = prepared
        .select(request(&prepared, TabContext::CssStops { origin: None }))
        .unwrap();
    let error = without_origin
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::MissingPhaseTwoContext(PhaseTwoContext::TabOrigin),
        })
    );
    // Root SPACE20 * numeric2 =40; from A10 the ordinary TAB reaches40,
    // B ends50, and one E10 Cell ends60. Its Inline SPACE10 keeps its own T20.
    let zero = complete(&prepared, TabContext::CssStops { origin: Some(0.0) });
    verify_mixed(&prepared, &zero, &bytes, 40.0, 50.0, 60.0);
    let five = complete(&prepared, TabContext::CssStops { origin: Some(5.0) });
    verify_mixed(&prepared, &five, &bytes, 45.0, 55.0, 65.0);
    drop(system);
    verify_mixed(
        &prepared,
        &zero.with_spacing(SpacingAllocations::None).unwrap(),
        &bytes,
        40.0,
        50.0,
        60.0,
    );
    verify_mixed(
        &prepared,
        &five.with_spacing(SpacingAllocations::None).unwrap(),
        &bytes,
        45.0,
        55.0,
        65.0,
    );
}

fn container_premises_for_scope(prepared: &PreparedText, capture: &Captured, bytes: &[u8]) {
    let scope = capture.processed.captured_scope(3).unwrap();
    let metrics = prepared.scope_metrics(&scope).unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(metrics.font_reference(), font_reference());
    assert_eq!(metrics.effective_instance().unwrap().face().bytes(), bytes);
}
