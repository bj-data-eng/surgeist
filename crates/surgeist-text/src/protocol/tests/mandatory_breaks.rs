use super::*;

fn with_forced_control(text: &str, at: usize) -> PreparedText {
    let mut system = system();
    let source = Source::new(text);
    let analysis = AnalyzedSource::from_paragraph(
        ParagraphInput {
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            source: source.clone(),
            base_style: style(),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            origins: vec![paragraph::ParagraphOrigin {
                text_node: None,
                source,
                local: crate::Range::new(0, text.encode_utf16().count()),
                aggregate: crate::Range::new(0, text.encode_utf16().count()),
                kind: paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![paragraph::ParagraphControl {
                at,
                kind: paragraph::ParagraphControlKind::ForcedBreak,
            }],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Control(0),
                affinity: crate::Affinity::After,
            }],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap()
}

fn selected_request(
    p: &PreparedText,
    start: PreparedBoundary,
    termination: PreparedOpportunity,
) -> LineSelectionRequest {
    let mut value = request(p);
    value.start = start;
    value.end = termination.view().boundary();
    let from = value.start.view().participant_offset();
    let to = value.end.view().participant_offset();
    value.participants = p.participants().skip(from).take(to - from).collect();
    value.controls = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(control) => Some({
                let occurrence = control.clone();
                ControlAllocation {
                    boundary: matches!(
                        occurrence.view().kind(),
                        crate::ParagraphControlKind::InlineBoundaryStart
                            | crate::ParagraphControlKind::InlineBoundaryEnd
                    )
                    .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                    occurrence,
                    strut: None,
                }
            }),
            _ => None,
        })
        .collect();
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = match termination.view().kind() {
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
        _ => unreachable!(),
    };
    value.termination = crate::PreparedTermination::Ordinary(termination);
    value
}

fn opportunity(p: &PreparedText, kind: PreparedBreakKind) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|view| view.kind() == kind)
        .unwrap()
        .handle()
}

fn reject_skipped_forced(p: PreparedText) {
    let value = selected_request(
        &p,
        p.boundary_views().next().unwrap().handle(),
        opportunity(&p, PreparedBreakKind::ParagraphEnd),
    );
    let error = p.select(value).unwrap_err();
    assert!(matches!(
        reason(&error),
        ProtocolError::IllegalOpportunity { .. }
    ));
}

fn finalized_forced(p: &PreparedText) -> FinalizedLine {
    p.select(selected_request(
        p,
        p.boundary_views().next().unwrap().handle(),
        opportunity(p, PreparedBreakKind::Forced),
    ))
    .unwrap()
    .finalize(crate::LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: crate::ScopeAllocations::None,
    })
    .unwrap()
}

fn placed_lines(
    p: PreparedText,
    coverage: PlacementCoverage,
    lines: Vec<FinalizedLine>,
) -> crate::Result<crate::Layout> {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 16.0 + index as f64 * 20.0,
                    },
                    line,
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
}

#[test]
fn source_forced_break_cannot_be_crossed_by_a_later_paragraph_end() {
    reject_skipped_forced(prepared(&mut system(), "A\nB", WhiteSpace::Preserve));
}

#[test]
fn native_forced_control_cannot_be_crossed_by_a_later_paragraph_end() {
    reject_skipped_forced(with_forced_control("AB", 1));
}

#[test]
fn terminal_source_forced_break_cannot_be_replaced_by_same_boundary_paragraph_end() {
    reject_skipped_forced(prepared(&mut system(), "A\n", WhiteSpace::Preserve));
}

#[test]
fn terminal_native_forced_control_cannot_be_replaced_by_same_boundary_paragraph_end() {
    reject_skipped_forced(with_forced_control("A", 1));
}

fn reject_missing_terminal_line(p: PreparedText) {
    let line = finalized_forced(&p);
    let error = placed_lines(p, PlacementCoverage::WholeParagraph, vec![line]).unwrap_err();
    assert!(matches!(
        reason(&error),
        ProtocolError::IncompleteLineCoverage { .. }
    ));
}

#[test]
fn whole_paragraph_requires_terminal_empty_line_after_source_forced_break() {
    reject_missing_terminal_line(prepared(&mut system(), "A\n", WhiteSpace::Preserve));
}

#[test]
fn whole_paragraph_requires_terminal_empty_line_after_native_forced_control() {
    reject_missing_terminal_line(with_forced_control("A", 1));
}

#[test]
fn bounded_coverage_can_end_at_the_actual_terminal_forced_boundary() {
    for p in [
        prepared(&mut system(), "A\n", WhiteSpace::Preserve),
        with_forced_control("A", 1),
    ] {
        let line = finalized_forced(&p);
        let selected = line.selected_line();
        let coverage = PlacementCoverage::Range {
            start: selected.request().start.clone(),
            end: selected.request().end.clone(),
        };
        assert_eq!(
            placed_lines(p, coverage, vec![line]).unwrap().lines().len(),
            1
        );
    }
}

#[test]
fn consumed_terminal_forced_boundary_accepts_a_real_empty_paragraph_end_line() {
    for p in [
        prepared(&mut system(), "A\n", WhiteSpace::Preserve),
        with_forced_control("A", 1),
    ] {
        let first = finalized_forced(&p);
        let last = p
            .select(selected_request(
                &p,
                first.selected_line().request().end.clone(),
                opportunity(&p, PreparedBreakKind::ParagraphEnd),
            ))
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert!(last.selected_line().request().participants.is_empty());
        assert_eq!(
            placed_lines(p, PlacementCoverage::WholeParagraph, vec![first, last])
                .unwrap()
                .lines()
                .len(),
            2
        );
    }
}

#[test]
fn authored_forced_break_stops_rtl_receiver_neutral_context() {
    const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
    fn prepare_rtl(forced: bool) -> PreparedText {
        let source = Source::new("A.A");
        let analyzed = AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: Style {
                    white_space: WhiteSpace::Preserve,
                    ..style()
                },
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::RightToLeft,
                paragraph_strut: ParagraphStrut::DefaultFont,
                origins: vec![paragraph::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: crate::Range::new(0, 3),
                    aggregate: crate::Range::new(0, 3),
                    kind: paragraph::ParagraphOriginKind::Authored,
                }],
                connections: Vec::new(),
                controls: forced
                    .then_some(paragraph::ParagraphControl {
                        at: 1,
                        kind: paragraph::ParagraphControlKind::ForcedBreak,
                    })
                    .into_iter()
                    .collect(),
                boundary_policies: Vec::new(),
                replacements: Vec::new(),
                anchor_order: forced
                    .then_some(paragraph::ParagraphAnchor {
                        item: paragraph::ParagraphAnchorItem::Control(0),
                        affinity: crate::Affinity::After,
                    })
                    .into_iter()
                    .collect(),
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        system()
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap()
    }

    for forced in [false, true] {
        let p = prepare_rtl(forced);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [65, 46, 65]
        );
        let levels = p
            .unit_views()
            .map(|unit| (unit.source_range(), unit.level().get()))
            .collect::<Vec<_>>();
        let ending = if forced {
            let control = p.control_views().next().unwrap();
            assert_eq!(
                control.kind(),
                &paragraph::ParagraphControlKind::ForcedBreak
            );
            assert_eq!(control.source_position().get(), 1);
            let offer = opportunity(&p, PreparedBreakKind::Forced);
            assert_eq!(offer.view().boundary().view().source_after().get(), 1);
            offer
        } else {
            assert!(p.control_views().next().is_none());
            assert!(
                !p.opportunity_views()
                    .any(|offer| offer.kind() == PreparedBreakKind::Forced)
            );
            opportunity(&p, PreparedBreakKind::ParagraphEnd)
        };
        let finish = |start, ending| {
            let mut request = selected_request(&p, start, ending);
            request.context.base_direction = ParagraphBaseDirection::RightToLeft;
            request.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
            p.select(request)
                .unwrap()
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap()
        };
        let first = finish(p.boundary_views().next().unwrap().handle(), ending);
        let mut lines = vec![first];
        if forced {
            lines.push(finish(
                lines[0].selected_line().request().end.clone(),
                opportunity(&p, PreparedBreakKind::ParagraphEnd),
            ));
        }
        let mut glyphs = Vec::new();
        for line in &lines {
            for run in line.run_views() {
                assert_eq!(run.face().bytes(), AHEM);
                assert_eq!(run.font_size(), 20.0);
                glyphs.extend(
                    run.glyphs()
                        .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
                );
            }
        }
        glyphs.sort_by_key(|(range, _, _)| range.start);
        assert_eq!(
            glyphs,
            [
                (crate::Range::new(0, 1), 35, 20.0),
                (crate::Range::new(1, 2), 16, 20.0),
                (crate::Range::new(2, 3), 35, 20.0),
            ]
        );
        let placed = placed_lines(p, PlacementCoverage::WholeParagraph, lines).unwrap();
        assert_eq!(placed.source().text().code_units(), [65, 46, 65]);
        assert_eq!(
            placed
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            if forced {
                vec![crate::Range::new(0, 1), crate::Range::new(1, 3)]
            } else {
                vec![crate::Range::new(0, 3)]
            }
        );
        // Pinned Writing Modes4 §2.4 (bidi-algo/HL1) keeps base RTL but
        // separates the UBA paragraphs at the authored B control. UAX9 rev52
        // P1 retains B with the first paragraph; receiver ON then has sos R
        // and following L. N2 resolves it R, so I2 leaves its level at1.
        // Without B, L ON L resolves ON to L by N1 and I2 gives level2.
        // https://www.unicode.org/reports/tr9/tr9-52.html#P1
        // https://www.unicode.org/reports/tr9/tr9-52.html#N2
        assert_eq!(
            levels,
            [
                (crate::Range::new(0, 1), 2),
                (crate::Range::new(1, 2), if forced { 1 } else { 2 }),
                (crate::Range::new(2, 3), 2),
            ],
            "authored ForcedBreak = {forced}"
        );
    }
}
