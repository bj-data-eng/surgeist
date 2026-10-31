//! Genuine nonformatted placement eligibility, coverage and source custody.
use super::*;
use crate::{Affinity, Cursor, Hit, LineHeight, Range, SourceProcessingEvent as E};
use std::sync::Arc;

fn nf_style(mode: WhiteSpace) -> Style {
    Style {
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: mode,
        wrap: crate::Wrap::Word,
        hyphens: crate::Hyphens::None,
        word_break: crate::WordBreak::Normal,
        word_space_transform: crate::WordSpaceTransform::None,
        ..style()
    }
}
fn nf_input(raw: &str, mode: WhiteSpace) -> crate::ContextualSourceInput {
    let source = Source::new(raw);
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    let origins = if raw.is_empty() {
        vec![]
    } else {
        events.push(E::Origin {
            origin: 0,
            affinity: Affinity::After,
        });
        vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(0, raw.encode_utf16().count()),
            aggregate: Range::new(0, raw.encode_utf16().count()),
            kind: crate::ParagraphOriginKind::Authored,
        }]
    };
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    crate::ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        source,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        envelope: crate::SourceProcessingEnvelope::Complete,
        origins,
        controls: vec![],
        connections: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        computed_line_boundaries: vec![],
        scopes: vec![crate::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: crate::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: nf_style(mode),
            used_direction: ParagraphBaseDirection::LeftToRight,
        }],
        paragraphs: vec![crate::SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events,
    }
}
fn nf_prepare(value: crate::ContextualSourceInput) -> PreparedText {
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let a = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = a
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| crate::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    let a = a.with_wrapping_policies(policies).unwrap();
    system().prepare(&a, PreparationOptions::default()).unwrap()
}
fn nf_allocation() -> LineBoxStrut {
    LineBoxStrut {
        scope: None,
        metrics: LineStrut {
            baseline: 12.0,
            line_extent: 20.0,
        },
        alignment: LineBoxAlignment::Baseline,
    }
}
fn nf_line(
    p: &PreparedText,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
    end: LineEnd,
) -> FinalizedLine {
    let mut r = request(p);
    let first = start.view().participant_offset();
    let last = offer.view().boundary().view().participant_offset();
    r.start = start;
    r.end = offer.view().boundary();
    r.termination = PreparedTermination::Ordinary(offer);
    r.participants = p.participants().skip(first).take(last - first).collect();
    r.controls = r
        .participants
        .iter()
        .filter_map(|part| match part {
            PreparedParticipant::Control(c) => Some(ControlAllocation {
                occurrence: c.clone(),
                strut: None,
                boundary: None,
            }),
            _ => None,
        })
        .collect();
    r.context.start = if first == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = end;
    // A real explicit caller allocation, separate from the actual Ahem default.
    r.line_box_struts = vec![nf_allocation()];
    p.select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
fn nf_whole(p: &PreparedText) -> FinalizedLine {
    let end = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    nf_line(
        p,
        p.boundary_views().next().unwrap().handle(),
        end,
        LineEnd::Paragraph,
    )
}
fn nf_place(p: &PreparedText, line: FinalizedLine, coverage: PlacementCoverage) -> crate::Layout {
    let capability = line.nonformatted().unwrap();
    assert_eq!(capability.finalized_line().feedback().line_height, 20.0);
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage,
        entries: vec![PlacementEntry::NonFormatted(NonFormattedLinePlacement {
            line: capability,
            origin: crate::Point::new(7.0, 11.0),
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}

#[test]
fn all_nonformatted_empty_and_removed_paragraphs_keep_nominal_metrics_without_geometry() {
    for (raw, mode) in [("", WhiteSpace::Collapse), ("  ", WhiteSpace::Discard)] {
        let p = nf_prepare(nf_input(raw, mode));
        let line = nf_whole(&p);
        assert_eq!(line.feedback().line_height, 20.0);
        assert_eq!(line.feedback().baseline, 12.0);
        assert!(!line.has_in_flow_content());
        let selected = line.selected_line();
        let layout = nf_place(&p, line, PlacementCoverage::WholeParagraph);
        assert_eq!(
            layout.analyzed_source().identity(),
            p.analyzed_source().identity()
        );
        assert_eq!(layout.source(), p.analyzed_source().source());
        assert_eq!(
            layout.source().text().code_units(),
            raw.encode_utf16().collect::<Vec<_>>()
        );
        assert!(layout.lines().is_empty());
        assert_eq!(layout.metrics().line_count(), 0);
        assert_eq!(layout.metrics().size(), crate::Size::new(0.0, 0.0));
        assert_eq!(layout.bounds(), crate::Rect::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(layout.metrics().first_baseline(), None);
        assert_eq!(layout.metrics().last_baseline(), None);
        assert_eq!(layout.positions().count(), 0);
        assert_eq!(layout.hit(crate::Point::new(7.0, 11.0)), Hit::None);
        let retained = layout.nonformatted_lines().collect::<Vec<_>>();
        assert_eq!(retained.len(), 1);
        assert_eq!(retained[0].origin, crate::Point::new(7.0, 11.0));
        assert_eq!(
            retained[0].line.finalized_line().feedback().line_height,
            20.0
        );
        assert_eq!(
            retained[0].line.finalized_line().selected_line().request(),
            selected.request()
        );
        if mode == WhiteSpace::Discard {
            let origin = p.origin_views().next().unwrap().handle();
            assert_eq!(origin.view().local_range(), Range::new(0, 2));
            assert_eq!(origin.view().aggregate_range(), Range::new(0, 2));
            let other = nf_prepare(nf_input("  ", WhiteSpace::Discard));
            let foreign = other.origin_views().next().unwrap().handle();
            assert_eq!(
                reason(
                    &layout
                        .source_range_fragments(&foreign, Range::new(0, 2))
                        .unwrap_err()
                ),
                &ProtocolError::ForeignPreparation {
                    kind: ProtocolHandleKind::Origin,
                    occurrence: 0,
                }
            );
            // Both original spaces are covered by the real retained receipt.
            // Removed source is valid custody, with no geometry/font lender.
            let observed = layout
                .source_range_fragments(&origin, Range::new(0, 2))
                .unwrap();
            assert_eq!(observed.origin(), &origin);
            assert_eq!(observed.local_range(), Range::new(0, 2));
            assert_eq!(observed.layout().key(), layout.key());
            assert_eq!(observed.units().len(), 0);
        }
        for index in [0, raw.len()] {
            assert_eq!(
                reason(
                    &layout
                        .cursor(Cursor::new(index, Affinity::After))
                        .unwrap_err()
                ),
                &ProtocolError::NoFormattedLineGeometry
            );
        }
        let invalid = layout
            .cursor(Cursor::new(raw.len() + 1, Affinity::After))
            .unwrap_err();
        assert_eq!(invalid.code, crate::ErrorCode::InvalidRange);
        assert!(
            matches!(invalid.detail(),Some(crate::ErrorDetail::InvalidSourceIndex{index,text_len,..})if *index==raw.len()+1&&*text_len==raw.len())
        );
    }
}

#[test]
fn nonformatted_eligibility_rejects_forced_text_and_real_zero_inflow_atomic() {
    let mut forced = nf_input("", WhiteSpace::Collapse);
    forced.controls = vec![crate::ParagraphControl {
        at: 0,
        kind: crate::ParagraphControlKind::ForcedBreak,
    }];
    forced.events.insert(
        2,
        E::Control {
            control: 0,
            affinity: Affinity::After,
        },
    );
    let p = nf_prepare(forced);
    let offer = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let line = nf_line(
        &p,
        p.boundary_views().next().unwrap().handle(),
        offer,
        LineEnd::Forced,
    );
    assert_eq!(
        reason(&line.nonformatted().unwrap_err()),
        &ProtocolError::NonFormattedForcedLine
    );
    let p = nf_prepare(nf_input("A", WhiteSpace::Preserve));
    assert_eq!(
        reason(&nf_whole(&p).nonformatted().unwrap_err()),
        &ProtocolError::NonFormattedInFlowContent
    );
    let mut input = nf_input("", WhiteSpace::Collapse);
    input.source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(659905),
        crate::InlineBoxKind::InFlow,
        0,
        crate::Size::new(0.0, 0.0),
    ));
    input.events.insert(
        2,
        E::Atomic {
            atomic: 0,
            affinity: Affinity::After,
        },
    );
    input.boundary_policies = [
        crate::ParagraphBoundaryDefinition::AtomicBefore(0),
        crate::ParagraphBoundaryDefinition::AtomicAfter(0),
    ]
    .into_iter()
    .map(|definition| crate::ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: crate::Wrap::Word,
        following_wrap: crate::Wrap::Word,
        ancestor_wrap: crate::Wrap::Word,
    })
    .collect();
    let p = nf_prepare(input);
    let mut r = request(&p);
    r.line_box_struts = vec![nf_allocation()];
    r.atomics = p
        .atomic_views()
        .map(|a| AtomicAllocation {
            occurrence: a.handle(),
            allocation: AtomicLineAllocation::InFlow {
                flow: AtomicFlowAllocation {
                    advance: 0.0,
                    before_baseline: 0.0,
                    after_baseline: 0.0,
                    occupied_start: 0.0,
                    occupied_end: 0.0,
                },
                box_geometry: AtomicBoxAllocation {
                    border: crate::Rect::new(0.0, 0.0, 0.0, 0.0),
                    content: crate::Rect::new(0.0, 0.0, 0.0, 0.0),
                    baseline: 0.0,
                },
                alignment: LineBoxAlignment::Baseline,
            },
        })
        .collect();
    let line = p
        .select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 0.0);
    assert_eq!(line.atomic_slots().count(), 1);
    assert_eq!(
        reason(&line.nonformatted().unwrap_err()),
        &ProtocolError::NonFormattedInFlowContent
    );
}

#[test]
fn removed_middle_soft_receipt_preserves_original_selection_between_real_visual_lines() {
    let mut input = nf_input("A  B", WhiteSpace::Collapse);
    let source = input.source.clone();
    input.origins = [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
        .into_iter()
        .map(|range| crate::ParagraphOrigin {
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: crate::ParagraphOriginKind::Authored,
            text_node: None,
        })
        .collect();
    input.connections = (0..2)
        .map(|i| crate::SourceProcessingConnection {
            before_origin: i,
            after_origin: i + 1,
            kind: crate::ParagraphConnectionKind::Continuous,
        })
        .collect();
    input.boundary_policies = (0..2)
        .map(|i| crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    input.controls = vec![crate::ParagraphControl {
        at: 1,
        kind: crate::ParagraphControlKind::Wbr {
            style: Box::new(nf_style(WhiteSpace::Collapse)),
        },
    }];
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Control {
            control: 0,
            affinity: Affinity::After,
        },
        E::Connection(0),
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::Connection(1),
        E::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let p = nf_prepare(input);
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        [65, 32, 66]
    );
    let control = p.control_views().next().unwrap().handle();
    assert!(matches!(
        control.view().kind(),
        crate::ParagraphControlKind::Wbr { .. }
    ));
    let wbr_index = p
        .participants()
        .position(|part| part == PreparedParticipant::Control(control.clone()))
        .unwrap();
    let first = p
        .opportunity_views()
        .find(|o| {
            o.kind() == PreparedBreakKind::Allowed
                && o.boundary().view().participant_offset() == wbr_index + 1
        })
        .expect("real authored wbr owns this actual mixed-prefix capability")
        .handle();
    assert_eq!(first.view().boundary().view().source_after().get(), 1);
    let middle = p
        .opportunity_views()
        .filter(|o| {
            o.kind() == PreparedBreakKind::Allowed
                && o.boundary().view().source_after().get() == 3
                && o.boundary().view().participant_offset()
                    > first.view().boundary().view().participant_offset()
        })
        .max_by_key(|o| o.boundary().view().participant_offset())
        .expect("normal collapsed SPACE must issue a real following soft capability")
        .handle();
    let end = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let a = nf_line(
        &p,
        p.boundary_views().next().unwrap().handle(),
        first.clone(),
        LineEnd::Soft,
    );
    let spaces = nf_line(&p, first.view().boundary(), middle.clone(), LineEnd::Soft);
    let b = nf_line(&p, middle.view().boundary(), end, LineEnd::Paragraph);
    assert!(!spaces.has_in_flow_content());
    assert_eq!(spaces.feedback().advance, 0.0);
    assert_eq!(spaces.feedback().line_height, 20.0);
    // A genuinely distinct selected interval establishes partial coverage;
    // source_after on the same empty Paragraph boundary cannot do that.
    let partial = nf_place(
        &p,
        spaces.clone(),
        PlacementCoverage::Range {
            start: first.view().boundary(),
            end: middle.view().boundary(),
        },
    );
    assert_eq!(
        reason(&partial.cursor(Cursor::new(0, Affinity::After)).unwrap_err()),
        &ProtocolError::OutsidePlacement {
            index: 0,
            coverage: Range::new(1, 3),
        }
    );
    assert_eq!(
        reason(&partial.cursor(Cursor::new(1, Affinity::After)).unwrap_err()),
        &ProtocolError::NoFormattedLineGeometry
    );
    let first_origin = p.origin_views().next().unwrap().handle();
    let outside = partial
        .source_range_fragments(&first_origin, Range::new(0, 1))
        .unwrap_err();
    assert!(matches!(
        outside.detail(),
        Some(crate::ErrorDetail::SourceRangeFragments {
            reason: crate::SourceRangeFragmentError::OutsidePlacement { aggregate }
        }) if *aggregate == Range::new(0, 1)
    ));
    let retained = spaces.nonformatted().unwrap();
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![
            PlacementEntry::Formatted(LinePlacement {
                line: a,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 12.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }),
            PlacementEntry::NonFormatted(NonFormattedLinePlacement {
                line: retained,
                origin: crate::Point::new(0.0, 20.0),
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }),
            PlacementEntry::Formatted(LinePlacement {
                line: b,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 32.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }),
        ],
    })
    .unwrap();
    assert_eq!(layout.lines().len(), 2);
    assert_eq!(layout.metrics().line_count(), 2);
    let retained = layout.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(retained.len(), 1);
    assert_eq!(
        retained[0]
            .line
            .finalized_line()
            .selected_line()
            .request()
            .context
            .end,
        LineEnd::Soft
    );
    assert_eq!(
        retained[0].line.finalized_line().feedback().line_height,
        20.0
    );
    let positions = layout.positions().collect::<Vec<_>>();
    let start = positions
        .iter()
        .find(|p| p.source_cursor() == Cursor::new(0, Affinity::After))
        .unwrap();
    let end = positions
        .iter()
        .find(|p| p.source_cursor() == Cursor::new(4, Affinity::Before))
        .unwrap();
    let selection = layout.position_selection(start, end).unwrap();
    assert!(!selection.is_collapsed());
    assert_eq!(selection.source_text().code_units(), [65, 32, 32, 66]);
    assert_eq!(selection.plaintext().code_units(), [65, 32, 66]);
    let origins = p.origin_views().map(|o| o.handle()).collect::<Vec<_>>();
    // The middle origin's original two spaces remain wholly accounted for,
    // while its selected collapsed representative was removed at the Soft edge.
    let removed = layout
        .source_range_fragments(&origins[1], Range::new(1, 3))
        .unwrap();
    assert_eq!(removed.origin(), &origins[1]);
    assert_eq!(removed.local_range(), Range::new(1, 3));
    assert_eq!(removed.layout().key(), layout.key());
    assert_eq!(removed.units().len(), 0);
    for (origin, local, line) in [
        (&origins[0], Range::new(0, 1), 0),
        (&origins[2], Range::new(3, 4), 1),
    ] {
        let visible = layout.source_range_fragments(origin, local).unwrap();
        assert_eq!(visible.origin(), origin);
        assert_eq!(visible.local_range(), local);
        assert_eq!(
            visible
                .units()
                .map(|unit| unit.line_index())
                .collect::<Vec<_>>(),
            vec![line]
        );
    }
    let pieces = selection.source_pieces();
    assert_eq!(pieces.len(), 3);
    for (i, expected) in [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
        .into_iter()
        .enumerate()
    {
        assert_eq!(pieces[i].origin(), &origins[i]);
        assert_eq!(pieces[i].local_range(), expected);
        assert_eq!(pieces[i].aggregate_range(), expected);
        assert_eq!(
            pieces[i].code_units(),
            &source.text().code_units()[expected.start..expected.end]
        );
    }
    assert!(selection.geometry().rects().iter().all(|r| r.line() < 2));
    for line in 0..2 {
        assert!(
            selection
                .geometry()
                .rects()
                .iter()
                .any(|r| r.line() == line)
        );
    }
    assert_eq!(
        layout
            .run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(35, 10.0), (36, 10.0)]
    );
}

#[test]
fn explicit_placement_key_tracks_alignment_and_replays_semantic_requests() {
    // Ahem's A is glyph 35, with 1000/1000 em advance: 10 at size 10.
    // The genuine LTR line has 40 - 10 = 30 units of alignment free space.
    let p = nf_prepare(nf_input("A", WhiteSpace::Preserve));
    let finalize_and_place = |alignment| {
        let mut r = request(&p);
        r.context.available_inline = Some(40.0);
        r.context.alignment = alignment;
        let line = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let layout = place(p.clone(), line.clone(), 7.0, 11.0).unwrap();
        (line, layout)
    };
    let (start_line, start) = finalize_and_place(crate::Alignment::Start);
    let (start_replay_line, start_replay) = finalize_and_place(crate::Alignment::Start);
    let (end_line, end) = finalize_and_place(crate::Alignment::End);
    let (end_replay_line, end_replay) = finalize_and_place(crate::Alignment::End);
    assert_eq!(
        start_line.selected_line().request(),
        start_replay_line.selected_line().request()
    );
    assert_eq!(
        end_line.selected_line().request(),
        end_replay_line.selected_line().request()
    );
    let mut end_request = end_line.selected_line().request().clone();
    end_request.context.alignment = crate::Alignment::Start;
    assert_eq!(&end_request, start_line.selected_line().request());
    for (line, layout, expected_x) in [
        (&start_line, &start, 7.0),
        (&start_replay_line, &start_replay, 7.0),
        (&end_line, &end, 37.0),
        (&end_replay_line, &end_replay, 37.0),
    ] {
        assert_eq!(line.prepared_text(), p);
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(
            layout.analyzed_source().identity(),
            p.analyzed_source().identity()
        );
        assert_eq!(layout.source(), p.analyzed_source().source());
        assert_eq!(layout.source().text().code_units(), &[65]);
        assert_eq!(layout.lines().len(), 1);
        assert_eq!(layout.lines()[0].range(), Range::new(0, 1));
        assert_eq!(
            line.run_views()
                .flat_map(|run| {
                    run.glyphs()
                        .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            vec![(Range::new(0, 1), 35, 10.0)]
        );
        let glyphs = layout
            .run_views()
            .flat_map(|run| {
                let origin = run.origin();
                run.glyphs()
                    .map(move |glyph| {
                        (
                            glyph.id(),
                            glyph.advance(),
                            origin.x + glyph.pen() + f64::from(glyph.x()),
                            origin.y,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, vec![(35, 10.0, expected_x, 11.0)]);
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            expected_x
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(1, Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            expected_x + 10.0
        );
    }
    // These are separately selected/finalized/placed semantic replays, not clones.
    assert_eq!(start.key(), start_replay.key());
    assert_eq!(end.key(), end_replay.key());
    assert_ne!(
        start.key(),
        end.key(),
        "different real alignment geometry needs distinct explicit placement keys"
    );
}
