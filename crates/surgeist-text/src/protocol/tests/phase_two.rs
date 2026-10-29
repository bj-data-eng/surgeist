//! Functional tests of new owner-issued measurements and terminal capture.
use super::*;
use crate::{Affinity, Cursor, Layout, ParagraphTerminalEdge as Terminal, Range};

fn input(text: &str, terminal: Terminal, mode: WhiteSpace) -> ParagraphInput {
    let source = Source::new(text);
    let end = source.text().len();
    ParagraphInput {
        terminal_edge: terminal,
        source: source.clone(),
        base_style: Style {
            size: 10.0,
            white_space: mode,
            ..style()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        tab_reference: None,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, end),
            aggregate: Range::new(0, end),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn prepare(system: &mut System, input: ParagraphInput) -> PreparedText {
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(
                    input,
                    crate::AnalysisTypography::HorizontalComposition,
                )
                .unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn line(p: &PreparedText, width: Option<f64>, indent: f64) -> crate::Result<FinalizedLine> {
    let mut r = request(p);
    r.context.available_inline = width;
    r.context.indent = indent;
    p.select(r)?.finalize(SpacingAllocations::None)
}

#[test]
fn actual_terminal_role_is_consumed_only_by_nonzero_eligible_preserved_edge() {
    let mut system = system();
    let p = prepare(
        &mut system,
        input("X ", Terminal::Unknown, WhiteSpace::Preserve),
    );
    assert_eq!(p.terminal_edge(), Terminal::Unknown);
    assert_eq!(
        reason(&line(&p, Some(40.0), 0.0).unwrap_err()),
        &ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
    );
    let p = prepare(
        &mut system,
        input("X ", Terminal::NonBlockBoundary, WhiteSpace::Preserve),
    );
    assert_eq!(
        reason(&line(&p, Some(40.0), 0.0).unwrap_err()),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::CssParagraphBoundary
        }
    );
    for (text, mode) in [
        ("X", WhiteSpace::Preserve),
        ("X\u{a0}", WhiteSpace::Preserve),
        ("X ", WhiteSpace::Collapse),
    ] {
        let p = prepare(&mut system, input(text, Terminal::Unknown, mode));
        assert!(line(&p, Some(40.0), 0.0).is_ok());
    }
    let mut nowrap = input("X ", Terminal::Unknown, WhiteSpace::Preserve);
    nowrap.base_style.wrap = crate::Wrap::None;
    let p = prepare(&mut system, nowrap);
    let actual = line(&p, Some(15.0), 0.0).unwrap();
    assert_eq!(
        (
            actual.feedback().advance,
            actual.measurements().fit().advance()
        ),
        (20.0, 20.0)
    );
    let mut zero = input(" ", Terminal::Unknown, WhiteSpace::Preserve);
    zero.base_style.word_spacing = -10.0;
    let p = prepare(&mut system, zero);
    assert_eq!(line(&p, None, 0.0).unwrap().feedback().advance, 0.0);
}

#[test]
fn zero_preserved_edges_consume_context_only_for_a_material_preceding_group() {
    let mut system = system();
    for terminal in [
        Terminal::Unknown,
        Terminal::NonBlockBoundary,
        Terminal::BlockEnd,
    ] {
        for preceding in [0.0, 10.0] {
            let mut captured = input("X\u{2003} ", terminal, WhiteSpace::Preserve);
            captured.source.span(
                Range::new(1, 2),
                Style {
                    white_space: WhiteSpace::Collapse,
                    ..captured.base_style.clone()
                },
            );
            captured.source.span(
                Range::new(2, 3),
                Style {
                    word_spacing: -10.0,
                    // X/EM has gap0. EM/SPACE has half this full value,
                    // charged to EM; SPACE's raw10 plus word−10 stays0.
                    letter_spacing: 2.0 * (preceding - 10.0),
                    ..captured.base_style.clone()
                },
            );
            let p = prepare(&mut system, captured);
            let result = line(&p, Some(40.0), 0.0);
            if preceding != 0.0 && terminal != Terminal::BlockEnd {
                let error = result.unwrap_err();
                let expected = if terminal == Terminal::Unknown {
                    ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
                } else {
                    ProtocolError::UnsupportedContext {
                        field: ProtocolContextField::CssParagraphBoundary,
                    }
                };
                assert_eq!(reason(&error), &expected);
            } else {
                let actual = result.unwrap();
                let raw: Vec<_> = actual
                    .run_views()
                    .flat_map(|run| {
                        run.glyphs()
                            .map(|glyph| {
                                (
                                    glyph.range(),
                                    glyph.id(),
                                    glyph.x(),
                                    glyph.y(),
                                    glyph.advance(),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                assert_eq!(
                    raw,
                    [
                        (Range::new(0, 1), 58, 0.0, 0.0, 10.0),
                        (Range::new(1, 2), 247, 0.0, 0.0, 10.0),
                        (Range::new(2, 3), 3, 0.0, 0.0, 10.0),
                    ]
                );
                let cells: Vec<_> = actual.edge_units().collect();
                assert_eq!(cells.len(), 3);
                assert_eq!(
                    cells.iter().map(|cell| cell.advance()).collect::<Vec<_>>(),
                    [10.0, preceding, 0.0]
                );
                for (index, cell) in cells.iter().enumerate() {
                    assert_eq!(
                        cell.source_pieces().collect::<Vec<_>>(),
                        [Range::new(index, index + 1)]
                    );
                    assert_ne!(cell.disposition(), crate::LineEdgeDisposition::Removed);
                }
                assert_eq!(actual.feedback().advance, 10.0 + preceding);
                assert_eq!(
                    actual.measurements().alignment().advance(),
                    10.0 + preceding
                );
                assert_eq!(
                    actual.measurements().max_content_contribution().advance(),
                    10.0 + preceding
                );
                assert_eq!(
                    actual.measurements().min_content_contribution().advance(),
                    10.0
                );
                assert_eq!(actual.feedback().hanging_end, 0.0);
                if preceding != 0.0 {
                    assert_eq!(
                        actual.edge_units().nth(1).unwrap().disposition(),
                        crate::LineEdgeDisposition::Unconditional {
                            side: crate::LineEdgeSide::End,
                            terminal: false
                        }
                    );
                }
            }
        }
        let mut sole = input(" ", terminal, WhiteSpace::Preserve);
        sole.base_style.word_spacing = -10.0;
        let p = prepare(&mut system, sole);
        assert_eq!(line(&p, None, 0.0).unwrap().feedback().advance, 0.0);
    }
}

#[test]
fn only_consumed_hanging_frames_require_checked_capacity_precision() {
    let mut system = system();
    let conditional = prepare(
        &mut system,
        input("X ", Terminal::BlockEnd, WhiteSpace::Preserve),
    );
    let retained = line(&conditional, Some(40.0), 0.0).unwrap();
    let error = line(&conditional, Some(1e308), 1.0).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            phase: crate::TextNumericPhase::Finalization,
            category: crate::TextNumericError::NonZeroLost,
            field: "Phase II available indentation",
            ..
        })
    ));
    assert_eq!(retained.feedback().advance, 20.0);
    assert_eq!(retained.measurements().alignment().advance(), 20.0);

    let mut zero = input(" ", Terminal::BlockEnd, WhiteSpace::Preserve);
    zero.base_style.word_spacing = -10.0;
    let zero = prepare(&mut system, zero);
    let actual = line(&zero, Some(1e308), 1.0).unwrap();
    assert_eq!(actual.feedback().advance, 0.0);
    let edge = actual.edge_units().next().unwrap();
    assert_eq!(
        (
            edge.advance(),
            edge.excluded_contribution(),
            edge.outward_overlap()
        ),
        (0.0, 0.0, 0.0)
    );
    assert_eq!(edge.bounds().origin.x, 1.0);
}

#[test]
fn allocated_zero_retains_natural_eligibility_and_real_source_geometry_both_directions() {
    let mut system = system();
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        let mut captured = input("X ", Terminal::BlockEnd, WhiteSpace::Preserve);
        captured.base_direction = direction;
        let p = prepare(&mut system, captured);
        let mut r = request(&p);
        r.context.base_direction = direction;
        r.context.line_box_direction = match direction {
            crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
            crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
        };
        r.context.available_inline = Some(15.0);
        r.context.alignment = crate::Alignment::Center;
        let selected = p.select(r).unwrap();
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(natural.feedback().advance, 20.0);
        assert_eq!(natural.measurements().alignment().advance(), 15.0);
        let slots: Vec<_> = selected.spacing_slots().collect();
        assert_eq!(slots.len(), 1);
        assert_eq!(
            slots[0]
                .pieces()
                .map(|piece| piece.owner())
                .collect::<Vec<_>>(),
            [crate::SpacingSlotOwner::Source {
                origin: selected
                    .prepared_text()
                    .origin_views()
                    .next()
                    .unwrap()
                    .handle(),
                local: Range::new(1, 2),
                aggregate: Range::new(1, 2),
            }]
        );
        let actual = selected
            .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                slot: slots[0].clone(),
                amount: -10.0,
            }]))
            .unwrap();
        assert_eq!(
            (
                actual.feedback().advance,
                actual.measurements().fit().advance(),
                actual.measurements().alignment().advance()
            ),
            (10.0, 10.0, 10.0)
        );
        let edge = actual.edge_units().last().unwrap();
        assert_eq!(
            edge.disposition(),
            crate::LineEdgeDisposition::Conditional {
                side: if direction == ParagraphBaseDirection::LeftToRight {
                    crate::LineEdgeSide::End
                } else {
                    crate::LineEdgeSide::Start
                },
                eligible: true
            }
        );
        assert_eq!(
            (
                edge.advance(),
                edge.excluded_contribution(),
                edge.outward_overlap()
            ),
            (0.0, 0.0, 0.0)
        );
        assert_eq!(edge.source_pieces().collect::<Vec<_>>(), [Range::new(1, 2)]);
        let point = if direction == ParagraphBaseDirection::LeftToRight {
            12.5
        } else {
            2.5
        };
        assert_eq!(
            (edge.bounds().origin.x, edge.bounds().size.width),
            (point, 0.0)
        );
        let origin = p.origin_views().next().unwrap().handle();
        let baseline = actual.feedback().baseline;
        let layout = Layout::from_finalized_lines(PlacementRequest {
            prepared: p,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line: actual,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap();
        assert_eq!(layout.source().text().code_units(), &[88, 32]);
        let fragments = layout
            .source_range_fragments(&origin, Range::new(1, 2))
            .unwrap();
        let units: Vec<_> = fragments.units().collect();
        assert_eq!(units.len(), 1);
        let pieces: Vec<_> = units[0].pieces().collect();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].advance_edges(), (point, point));
        assert_eq!(pieces[0].effective_instance().font_size(), 10.0);
        assert_eq!(natural.feedback().advance, 20.0);
    }
}

#[test]
fn measurements_borrow_exact_final_owner_and_separate_all_purposes_and_indent() {
    let mut system = system();
    let p = prepare(
        &mut system,
        input("X ", Terminal::BlockEnd, WhiteSpace::Preserve),
    );
    let actual = line(&p, Some(20.0), 5.0).unwrap();
    let m = actual.measurements();
    assert_eq!(actual.feedback().advance, 20.0);
    assert_eq!(
        (
            m.fit().advance(),
            m.min_content_contribution().advance(),
            m.max_content_contribution().advance(),
            m.alignment().advance()
        ),
        (10.0, 10.0, 20.0, 15.0)
    );
    assert_eq!(
        (m.fit().occupied_start(), m.fit().occupied_end()),
        (0.0, 10.0)
    );
    assert_eq!(
        (m.alignment().occupied_start(), m.alignment().occupied_end()),
        (0.0, 15.0)
    );
    assert_eq!(actual.metric_bounds().size.width, 20.0);
    let rows: Vec<_> = actual.edge_units().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[1].disposition(),
        crate::LineEdgeDisposition::Conditional {
            side: crate::LineEdgeSide::End,
            eligible: true
        }
    );
    assert_eq!(
        (rows[1].excluded_contribution(), rows[1].outward_overlap()),
        (5.0, 5.0)
    );
    assert_eq!(
        rows[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    assert_eq!(
        rows[1].presentation_range(),
        PresentationRange { start: 1, end: 2 }
    );
    assert!(std::ptr::eq(rows[1].finalized_line(), &actual));
    let owners: Vec<_> = rows[1].owners().collect();
    assert!(matches!(
        &owners[..],
        [crate::LineEdgeOwner::Participant(
            PreparedParticipant::Text(_)
        )]
    ));
    let unconstrained = line(&p, None, 0.0).unwrap();
    assert_eq!(
        (
            unconstrained
                .measurements()
                .min_content_contribution()
                .advance(),
            unconstrained
                .measurements()
                .max_content_contribution()
                .advance(),
            unconstrained.metric_bounds().size.width
        ),
        (10.0, 20.0, 20.0)
    );
    assert_eq!(actual.measurements().alignment().advance(), 15.0);
}

#[test]
fn partial_terminal_group_exclusion_is_distinct_from_physical_overlap() {
    let mut system = system();
    let mut captured = input("X\u{2003} ", Terminal::BlockEnd, WhiteSpace::Preserve);
    captured.source.span(
        Range::new(1, 2),
        Style {
            white_space: WhiteSpace::Collapse,
            ..captured.base_style.clone()
        },
    );
    let p = prepare(&mut system, captured);
    let actual = line(&p, Some(25.0), 0.0).unwrap();
    assert_eq!(actual.feedback().advance, 30.0);
    let m = actual.measurements();
    assert_eq!(
        (
            m.fit().advance(),
            m.min_content_contribution().advance(),
            m.max_content_contribution().advance(),
            m.alignment().advance()
        ),
        (10.0, 10.0, 30.0, 15.0)
    );
    let rows: Vec<_> = actual.edge_units().collect();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[1].disposition(),
        crate::LineEdgeDisposition::Unconditional {
            side: crate::LineEdgeSide::End,
            terminal: true
        }
    );
    assert_eq!(
        (
            rows[1].excluded_contribution(),
            rows[1].outward_overlap(),
            rows[2].excluded_contribution(),
            rows[2].outward_overlap()
        ),
        (10.0, 0.0, 5.0, 5.0)
    );
    assert_eq!(
        rows[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    assert_eq!(
        rows[2].source_pieces().collect::<Vec<_>>(),
        [Range::new(2, 3)]
    );
    assert_eq!((rows[1].advance(), rows[2].advance()), (10.0, 10.0));
    assert_eq!(
        (rows[1].bounds().origin.x, rows[2].bounds().origin.x),
        (10.0, 20.0)
    );
}

#[test]
fn changed_terminal_capture_cannot_reuse_old_preparation_handles_or_result() {
    let mut system = system();
    let input = input("X ", Terminal::BlockEnd, WhiteSpace::Preserve);
    let old = prepare(&mut system, input.clone());
    let old_result = line(&old, Some(15.0), 0.0).unwrap();
    let mut changed = input;
    changed.terminal_edge = Terminal::Unknown;
    let new = prepare(&mut system, changed);
    assert_ne!(old, new);
    assert!(matches!(
        reason(&new.select(request(&old)).unwrap_err()),
        ProtocolError::ForeignPreparation { .. }
    ));
    assert!(line(&new, Some(15.0), 0.0).is_err());
    assert_eq!(old_result.measurements().alignment().advance(), 15.0);
    assert_eq!(old_result.feedback().hanging_end, 5.0);
}

#[test]
fn preserve_spaces_uses_its_real_processed_space_without_inventing_a_forced_end() {
    let mut system = system();
    for text in ["X\t", "X\n"] {
        let p = prepare(
            &mut system,
            input(text, Terminal::BlockEnd, WhiteSpace::PreserveSpaces),
        );
        let actual = line(&p, Some(15.0), 0.0).unwrap();
        assert_eq!(
            actual.selected_line().request().context.end,
            LineEnd::Paragraph
        );
        assert_eq!(
            (
                actual.feedback().advance,
                actual.measurements().fit().advance(),
                actual.measurements().alignment().advance(),
                actual.feedback().hanging_end
            ),
            (20.0, 10.0, 15.0, 5.0)
        );
        let last = actual.edge_units().last().unwrap();
        assert_eq!(last.source_pieces().collect::<Vec<_>>(), [Range::new(1, 2)]);
        let layout = Layout::from_finalized_lines(PlacementRequest {
            prepared: p,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line: actual,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 8.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap();
        assert_eq!(
            layout.source().text().code_units(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(2, Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            20.0
        );
    }
}

#[test]
fn omitted_soft_space_inside_wide_frame_has_no_physical_outward_overlap() {
    let mut system = system();
    let p = prepare(
        &mut system,
        input("X Y", Terminal::Unknown, WhiteSpace::Preserve),
    );
    let ending = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Allowed)
        .unwrap()
        .handle();
    let mut r = request(&p);
    r.end = ending.view().boundary();
    r.participants = p
        .participants()
        .take(r.end.view().participant_offset())
        .collect();
    r.context.end = LineEnd::Soft;
    r.context.available_inline = Some(40.0);
    r.termination = PreparedTermination::Ordinary(ending);
    let actual = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        (
            actual.feedback().advance,
            actual.measurements().fit().advance(),
            actual.measurements().alignment().advance()
        ),
        (20.0, 10.0, 10.0)
    );
    let edge = actual.edge_units().last().unwrap();
    assert_eq!(
        (
            edge.advance(),
            edge.excluded_contribution(),
            edge.outward_overlap()
        ),
        (10.0, 10.0, 0.0)
    );
    assert_eq!(
        edge.bounds(),
        crate::Rect::new(
            10.0,
            actual.metric_bounds().origin.y,
            10.0,
            actual.metric_bounds().size.height
        )
    );
    let owners: Vec<_> = edge.owners().collect();
    let [crate::LineEdgeOwner::Participant(PreparedParticipant::Text(unit))] = &owners[..] else {
        panic!("actual text owner");
    };
    let origins: Vec<_> = unit.view().origins().map(|o| o.origin()).collect();
    assert_eq!(origins, [p.origin_views().next().unwrap().handle()]);
}

#[test]
fn selected_replacement_space_and_css_tab_keep_alternative_custody_and_geometry() {
    let mut system = system();
    for replacement in [" ", "\t"] {
        let mut native = input("XX\u{ad}YYY", Terminal::BlockEnd, WhiteSpace::Preserve);
        native.base_style.tab_size = crate::TabSize::spaces(4.0).unwrap();
        native.tab_reference = Some(
            crate::ParagraphTabReference::new(
                native.base_style.clone(),
                ParagraphBaseDirection::LeftToRight,
            )
            .unwrap(),
        );
        native.replacements.push(
            crate::ParagraphReplacement::resolved_native_logical_before(
                crate::ParagraphReplacementDefinition::Source {
                    origin: 0,
                    source: Range::new(2, 3),
                },
                3,
                vec![replacement.into()],
            )
            .unwrap(),
        );
        let analyzed = crate::test_wrapping::bind_missing(
            &AnalyzedSource::from_paragraph(
                native,
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap(),
            crate::Wrap::Word,
        );
        let p = system
            .prepare(
                &analyzed,
                PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops).unwrap(),
            )
            .unwrap();
        let alternative = p.alternative_views().next().unwrap().handle();
        let offer = alternative.view().opportunity().unwrap();
        let end = offer.view().boundary();
        let mut r = request(&p);
        r.end = end.clone();
        r.termination = PreparedTermination::Ordinary(offer);
        r.alternative = Some(alternative.clone());
        r.context.end = LineEnd::Soft;
        r.context.available_inline = Some(80.0);
        r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
        r.participants = p
            .participants()
            .take(end.view().participant_offset())
            .collect();
        let line = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        let advance = if replacement == " " { 10.0 } else { 20.0 };
        assert_eq!(
            (line.feedback().advance, line.measurements().fit().advance()),
            (20.0 + advance, 20.0)
        );
        let row = line.edge_units().last().unwrap();
        assert_eq!(
            (
                row.advance(),
                row.excluded_contribution(),
                row.outward_overlap()
            ),
            (advance, advance, 0.0)
        );
        assert!(row.source_pieces().next().is_none());
        assert_eq!(
            row.owners().collect::<Vec<_>>(),
            [crate::LineEdgeOwner::Alternative(alternative.clone())]
        );
        assert!(
            matches!(alternative.view().definition(), crate::PreparedAlternativeDefinition::Source { source, .. } if source == Range::new(2, 3))
        );
        let original = p.origin_views().next().unwrap().handle();
        let baseline = line.feedback().baseline;
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: p.clone(),
            coverage: PlacementCoverage::Range {
                start: p.boundary_views().next().unwrap().handle(),
                end,
            },
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap();
        assert_eq!(
            layout.source().text().code_units(),
            &[88, 88, 173, 89, 89, 89]
        );
        let fragments = layout
            .source_range_fragments(&original, Range::new(2, 3))
            .unwrap();
        let units: Vec<_> = fragments.units().collect();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].alternatives().next(), Some(alternative));
        assert_eq!(
            units[0]
                .pieces()
                .map(|p| p.advance_edges())
                .collect::<Vec<_>>(),
            [(20.0, 20.0 + advance)]
        );
    }
}

fn zero_preserved_selection(
    system: &mut System,
    terminal: Terminal,
    direction: ParagraphBaseDirection,
) -> SelectedLine {
    let mut captured = input(" ", terminal, WhiteSpace::Preserve);
    captured.base_style.word_spacing = -10.0;
    captured.base_direction = direction;
    let p = prepare(system, captured);
    let mut r = request(&p);
    r.context.base_direction = direction;
    r.context.line_box_direction = match direction {
        crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
        crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
    };
    r.context.available_inline = Some(15.0);
    p.select(r).unwrap()
}

#[test]
fn unallocated_zero_terminal_is_neutral_but_known_block_retains_classification() {
    let mut system = system();
    for terminal in [
        Terminal::Unknown,
        Terminal::NonBlockBoundary,
        Terminal::BlockEnd,
    ] {
        let selected =
            zero_preserved_selection(&mut system, terminal, ParagraphBaseDirection::LeftToRight);
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(
            (
                natural.feedback().advance,
                natural.measurements().fit().advance(),
                natural.measurements().min_content_contribution().advance()
            ),
            (0.0, 0.0, 0.0)
        );
        let slots: Vec<_> = selected.spacing_slots().collect();
        assert_eq!(slots.len(), 1);
        assert_eq!(
            slots[0]
                .pieces()
                .map(|piece| piece.owner())
                .collect::<Vec<_>>(),
            [crate::SpacingSlotOwner::Source {
                origin: selected
                    .prepared_text()
                    .origin_views()
                    .next()
                    .unwrap()
                    .handle(),
                local: Range::new(0, 1),
                aggregate: Range::new(0, 1),
            }]
        );
        let edge = natural.edge_units().next().unwrap();
        assert_eq!(edge.source_pieces().collect::<Vec<_>>(), [Range::new(0, 1)]);
        if terminal != Terminal::BlockEnd {
            assert_eq!(
                edge.disposition(),
                crate::LineEdgeDisposition::UnconsumedTerminal {
                    side: crate::LineEdgeSide::End
                }
            );
        }
        if terminal == Terminal::BlockEnd {
            let allocated = selected
                .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                    slot: slots[0].clone(),
                    amount: 10.0,
                }]))
                .unwrap();
            assert_eq!(
                (
                    allocated.feedback().advance,
                    allocated.measurements().fit().advance(),
                    allocated
                        .measurements()
                        .min_content_contribution()
                        .advance(),
                    allocated
                        .measurements()
                        .max_content_contribution()
                        .advance(),
                    allocated.measurements().alignment().advance()
                ),
                (10.0, 0.0, 0.0, 10.0, 10.0)
            );
            assert_eq!(
                allocated.edge_units().next().unwrap().disposition(),
                crate::LineEdgeDisposition::Conditional {
                    side: crate::LineEdgeSide::End,
                    eligible: false
                }
            );
        }
    }
}

fn expanded_zero_terminal_error(
    terminal: Terminal,
    amount: f64,
    direction: ParagraphBaseDirection,
) -> crate::Error {
    let mut system = system();
    let selected = zero_preserved_selection(&mut system, terminal, direction);
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 0.0);
    let slot = selected.spacing_slots().next().unwrap();
    assert_eq!(
        slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
        [crate::SpacingSlotOwner::Source {
            origin: selected
                .prepared_text()
                .origin_views()
                .next()
                .unwrap()
                .handle(),
            local: Range::new(0, 1),
            aggregate: Range::new(0, 1),
        }]
    );
    selected
        .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
            slot,
            amount,
        }]))
        .expect_err("expanded preserved source consumes actual terminal authority")
}

#[test]
fn expanding_unknown_zero_preserved_edge_requires_terminal_authority() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        for amount in [-10.0, 10.0] {
            assert_eq!(
                reason(&expanded_zero_terminal_error(
                    Terminal::Unknown,
                    amount,
                    direction
                )),
                &ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
            );
        }
    }
}

#[test]
fn expanding_nonblock_zero_preserved_edge_cannot_invent_css_block_end() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        for amount in [-10.0, 10.0] {
            assert_eq!(
                reason(&expanded_zero_terminal_error(
                    Terminal::NonBlockBoundary,
                    amount,
                    direction
                )),
                &ProtocolError::UnsupportedContext {
                    field: ProtocolContextField::CssParagraphBoundary
                }
            );
        }
    }
}

#[test]
fn deferred_zero_preserved_edge_tracks_its_exact_earlier_group_not_ordinary_prefix() {
    let mut system = system();
    for terminal in [
        Terminal::Unknown,
        Terminal::NonBlockBoundary,
        Terminal::BlockEnd,
    ] {
        let mut captured = input("X  ", terminal, WhiteSpace::Preserve);
        captured.base_style.word_spacing = -10.0;
        captured.source.span(
            Range::new(1, 2),
            Style {
                white_space: WhiteSpace::Collapse,
                ..captured.base_style.clone()
            },
        );
        let p = prepare(&mut system, captured);
        let selected = p.select(request(&p)).unwrap();
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(
            (
                natural.feedback().advance,
                natural.measurements().alignment().advance()
            ),
            (10.0, 10.0)
        );
        let slots: Vec<_> = selected.spacing_slots().collect();
        assert_eq!(slots.len(), 2);
        for (slot, range) in slots.iter().zip([Range::new(1, 2), Range::new(2, 3)]) {
            assert_eq!(
                slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
                [crate::SpacingSlotOwner::Source {
                    origin: selected
                        .prepared_text()
                        .origin_views()
                        .next()
                        .unwrap()
                        .handle(),
                    local: range,
                    aggregate: range,
                }]
            );
        }
        let unchanged = selected
            .finalize(SpacingAllocations::Complete(
                slots
                    .iter()
                    .map(|slot| SpacingAllocation {
                        slot: slot.clone(),
                        amount: 0.0,
                    })
                    .collect(),
            ))
            .unwrap();
        assert_eq!(unchanged.feedback().advance, 10.0);
        let expanded = selected.finalize(SpacingAllocations::Complete(vec![
            SpacingAllocation {
                slot: slots[0].clone(),
                amount: 10.0,
            },
            SpacingAllocation {
                slot: slots[1].clone(),
                amount: 0.0,
            },
        ]));
        if terminal == Terminal::BlockEnd {
            let expanded = expanded.unwrap();
            assert_eq!(
                (
                    expanded.feedback().advance,
                    expanded.measurements().fit().advance(),
                    expanded.measurements().min_content_contribution().advance(),
                    expanded.measurements().max_content_contribution().advance(),
                    expanded.measurements().alignment().advance()
                ),
                (20.0, 20.0, 10.0, 20.0, 20.0)
            );
        } else {
            let error =
                expanded.expect_err("earlier allocated group consumes deferred terminal authority");
            let expected = if terminal == Terminal::Unknown {
                ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
            } else {
                ProtocolError::UnsupportedContext {
                    field: ProtocolContextField::CssParagraphBoundary,
                }
            };
            assert_eq!(reason(&error), &expected);
        }
        assert_eq!(natural.feedback().advance, 10.0);
    }
}

#[test]
fn allocation_outside_deferred_terminal_dependencies_remains_admitted() {
    let mut system = system();
    for terminal in [Terminal::Unknown, Terminal::NonBlockBoundary] {
        let mut captured = input(" X ", terminal, WhiteSpace::Preserve);
        captured.base_style.word_spacing = -10.0;
        let p = prepare(&mut system, captured);
        let selected = p.select(request(&p)).unwrap();
        let slots: Vec<_> = selected.spacing_slots().collect();
        assert_eq!(slots.len(), 2);
        for (slot, range) in slots.iter().zip([Range::new(0, 1), Range::new(2, 3)]) {
            assert_eq!(
                slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
                [crate::SpacingSlotOwner::Source {
                    origin: selected
                        .prepared_text()
                        .origin_views()
                        .next()
                        .unwrap()
                        .handle(),
                    local: range,
                    aggregate: range,
                }]
            );
        }
        let actual = selected
            .finalize(SpacingAllocations::Complete(vec![
                SpacingAllocation {
                    slot: slots[0].clone(),
                    amount: 10.0,
                },
                SpacingAllocation {
                    slot: slots[1].clone(),
                    amount: 0.0,
                },
            ]))
            .unwrap();
        assert_eq!(
            (
                actual.feedback().advance,
                actual.measurements().fit().advance(),
                actual.measurements().min_content_contribution().advance(),
                actual.measurements().max_content_contribution().advance(),
                actual.measurements().alignment().advance()
            ),
            (20.0, 20.0, 20.0, 20.0, 20.0)
        );
        assert_eq!(
            actual.edge_units().last().unwrap().disposition(),
            crate::LineEdgeDisposition::UnconsumedTerminal {
                side: crate::LineEdgeSide::End
            }
        );
    }
}

#[test]
fn cancellation_between_deferred_rows_cannot_hide_terminal_dependence() {
    let mut system = system();
    for terminal in [Terminal::Unknown, Terminal::NonBlockBoundary] {
        let mut captured = input("X  ", terminal, WhiteSpace::Preserve);
        captured.base_style.word_spacing = -10.0;
        captured.source.span(
            Range::new(1, 2),
            Style {
                white_space: WhiteSpace::Collapse,
                ..captured.base_style.clone()
            },
        );
        let p = prepare(&mut system, captured);
        let selected = p.select(request(&p)).unwrap();
        let retained = selected.finalize(SpacingAllocations::None).unwrap();
        let slots: Vec<_> = selected.spacing_slots().collect();
        assert_eq!(slots.len(), 2);
        let error = selected
            .finalize(SpacingAllocations::Complete(vec![
                SpacingAllocation {
                    slot: slots[0].clone(),
                    amount: 10.0,
                },
                SpacingAllocation {
                    slot: slots[1].clone(),
                    amount: -10.0,
                },
            ]))
            .expect_err("net zero allocations still change exact terminal dependencies");
        let expected = if terminal == Terminal::Unknown {
            ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
        } else {
            ProtocolError::UnsupportedContext {
                field: ProtocolContextField::CssParagraphBoundary,
            }
        };
        assert_eq!(reason(&error), &expected);
        assert_eq!(retained.feedback().advance, 10.0);
        assert_eq!(
            retained
                .edge_units()
                .filter(|row| matches!(
                    row.disposition(),
                    crate::LineEdgeDisposition::UnconsumedTerminal { .. }
                ))
                .count(),
            2
        );
    }
}

#[test]
fn forced_zero_point_outside_frame_cannot_make_preceding_group_terminal() {
    let mut system = system();
    let mut captured = input("X\u{2003} \nY", Terminal::Unknown, WhiteSpace::Preserve);
    captured.source.span(
        Range::new(1, 2),
        Style {
            white_space: WhiteSpace::Collapse,
            ..captured.base_style.clone()
        },
    );
    captured.source.span(
        Range::new(2, 3),
        Style {
            word_spacing: -10.0,
            ..captured.base_style.clone()
        },
    );
    let p = prepare(&mut system, captured);
    let ending = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let mut r = request(&p);
    r.end = ending.view().boundary();
    r.participants = p
        .participants()
        .take(r.end.view().participant_offset())
        .collect();
    r.context.end = LineEnd::Forced;
    r.context.available_inline = Some(15.0);
    r.termination = PreparedTermination::Ordinary(ending);
    let selected = p.select(r).unwrap();
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(
        (
            natural.feedback().advance,
            natural.measurements().fit().advance(),
            natural.measurements().alignment().advance()
        ),
        (20.0, 20.0, 20.0)
    );
    assert_eq!(
        natural.edge_units().nth(1).unwrap().disposition(),
        crate::LineEdgeDisposition::Unconditional {
            side: crate::LineEdgeSide::End,
            terminal: false
        }
    );
    assert_eq!(
        natural.edge_units().nth(2).unwrap().disposition(),
        crate::LineEdgeDisposition::Conditional {
            side: crate::LineEdgeSide::End,
            eligible: false
        }
    );
    let slots: Vec<_> = selected.spacing_slots().collect();
    let actual = selected
        .finalize(SpacingAllocations::Complete(
            slots
                .iter()
                .map(|slot| SpacingAllocation {
                    slot: slot.clone(),
                    amount: if slot.pieces().any(|piece| matches!(piece.owner(), crate::SpacingSlotOwner::Source { aggregate, .. } if aggregate == Range::new(2, 3))) {
                        10.0
                    } else {
                        0.0
                    },
                })
                .collect(),
        ))
        .unwrap();
    assert_eq!(
        (
            actual.feedback().advance,
            actual.measurements().fit().advance(),
            actual.measurements().alignment().advance()
        ),
        (30.0, 20.0, 30.0)
    );
    assert_eq!(
        actual.edge_units().nth(1).unwrap().disposition(),
        natural.edge_units().nth(1).unwrap().disposition()
    );
    assert_eq!(
        actual
            .edge_units()
            .nth(2)
            .unwrap()
            .source_pieces()
            .collect::<Vec<_>>(),
        [Range::new(2, 3)]
    );
}

#[test]
fn deferred_terminal_failure_keeps_real_atomic_allocation_and_prior_result_retryable() {
    let mut system = system();
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        let mut captured = input(" ", Terminal::Unknown, WhiteSpace::Preserve);
        captured.base_direction = direction;
        captured.base_style.word_spacing = -10.0;
        captured.source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(91),
            crate::InlineBoxKind::InFlow,
            0,
            crate::Size::new(10.0, 10.0),
        ));
        captured.anchor_order.push(paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Atomic(0),
            affinity: Affinity::Before,
        });
        for definition in [
            paragraph::ParagraphBoundaryDefinition::AtomicBefore(0),
            paragraph::ParagraphBoundaryDefinition::AtomicAfter(0),
        ] {
            captured
                .boundary_policies
                .push(paragraph::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                });
        }
        let p = prepare(&mut system, captured);
        let mut r = request(&p);
        r.context.base_direction = direction;
        r.context.line_box_direction = match direction {
            crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
            crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
        };
        r.atomics = p
            .atomic_views()
            .map(|atomic| AtomicAllocation {
                occurrence: atomic.handle(),
                allocation: AtomicLineAllocation::InFlow {
                    flow: AtomicFlowAllocation {
                        advance: 10.0,
                        before_baseline: 8.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 10.0,
                    },
                    box_geometry: AtomicBoxAllocation {
                        border: crate::Rect::new(0.0, -8.0, 10.0, 10.0),
                        content: crate::Rect::new(0.0, -8.0, 10.0, 10.0),
                        baseline: 0.0,
                    },
                    alignment: LineBoxAlignment::Baseline,
                },
            })
            .collect();
        let selected = p.select(r).unwrap();
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(
            (
                natural.feedback().advance,
                natural.measurements().fit().advance()
            ),
            (10.0, 10.0)
        );
        let slot = selected.spacing_slots().next().unwrap();
        let error = selected
            .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                slot: slot.clone(),
                amount: 10.0,
            }]))
            .unwrap_err();
        assert_eq!(
            reason(&error),
            &ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
        );
        let retry = selected
            .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                slot,
                amount: 0.0,
            }]))
            .unwrap();
        assert_eq!(
            (natural.feedback().advance, retry.feedback().advance),
            (10.0, 10.0)
        );
        assert_eq!(
            retry
                .edge_units()
                .next()
                .unwrap()
                .source_pieces()
                .collect::<Vec<_>>(),
            [Range::new(0, 1)]
        );
    }
}

#[test]
fn genuine_soft_zero_preserved_edge_uses_its_break_without_terminal_authority() {
    let mut system = system();
    let mut captured = input("X Y", Terminal::Unknown, WhiteSpace::Preserve);
    captured.base_style.word_spacing = -10.0;
    let p = prepare(&mut system, captured);
    let ending = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Allowed)
        .unwrap()
        .handle();
    let mut r = request(&p);
    r.end = ending.view().boundary();
    r.participants = p
        .participants()
        .take(r.end.view().participant_offset())
        .collect();
    r.context.end = LineEnd::Soft;
    r.context.available_inline = Some(40.0);
    r.termination = PreparedTermination::Ordinary(ending);
    let selected = p.select(r).unwrap();
    let slot = selected.spacing_slots().next().unwrap();
    let actual = selected
        .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
            slot,
            amount: 10.0,
        }]))
        .unwrap();
    assert_eq!(
        (
            actual.feedback().advance,
            actual.measurements().fit().advance(),
            actual.measurements().alignment().advance()
        ),
        (20.0, 10.0, 10.0)
    );
    assert_eq!(
        actual.edge_units().last().unwrap().disposition(),
        crate::LineEdgeDisposition::Unconditional {
            side: crate::LineEdgeSide::End,
            terminal: true
        }
    );
}
