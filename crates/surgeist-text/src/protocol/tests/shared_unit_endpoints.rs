//! Physical endpoint chronology is distinct from a permission to divide a unit.

use super::*;
use crate::{
    Affinity, ContextualSourceInput, DocumentSegmentBreaks, Id, LineBreak, LineHeight,
    ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnectionKind,
    ParagraphControl, ParagraphControlKind, ParagraphOrigin, ParagraphOriginKind, Range,
    SourceProcessingConnection, SourceProcessingEnvelope, SourceProcessingEvent as E,
    SourceProcessingParagraph, SourceProcessingScope, SourceProcessingScopeBoundaries,
    SourceProcessingTopology, WordBreak, Wrap,
};
use std::sync::Arc;

fn capture(endpoints: bool, anywhere: bool) -> PreparedText {
    let base = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        word_break: if anywhere {
            WordBreak::Normal
        } else {
            WordBreak::BreakAll
        },
        line_break: if anywhere {
            LineBreak::Anywhere
        } else {
            LineBreak::Normal
        },
        wrap: Wrap::Word,
        ..Style::default()
    };
    let originals = [
        Source::identified("A", Id::from_u64(1_507_001), 11),
        Source::identified("\u{301}B", Id::from_u64(1_507_002), 13),
    ];
    // One actual Inline encloses only A. The following mark and B are an
    // ordinary connected original outside that Inline, with identical glyph
    // formatting. The End event is authored between A and its attached mark.
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::Control {
            control: 0,
            affinity: Affinity::After,
        },
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Control {
            control: 1,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::Connection(0),
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    if !endpoints {
        events.retain(|event| !matches!(event, E::Control { .. }));
    }
    let processed = Arc::new(
        crate::process_contextual_source(ContextualSourceInput {
            scope_alignment_limits: None,
            source: Source::identified("A\u{301}B", Id::from_u64(1_507_000), 7),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: originals
                .into_iter()
                .zip([Range::new(0, 1), Range::new(1, 3)])
                .map(|(source, aggregate)| ParagraphOrigin {
                    text_node: None,
                    local: Range::new(0, source.text().len()),
                    source,
                    aggregate,
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: ParagraphConnectionKind::Continuous,
            }],
            controls: if endpoints {
                vec![
                    ParagraphControl {
                        at: 0,
                        kind: ParagraphControlKind::InlineBoundaryStart,
                    },
                    ParagraphControl {
                        at: 1,
                        kind: ParagraphControlKind::InlineBoundaryEnd,
                    },
                ]
            } else {
                vec![]
            },
            boundary_policies: vec![ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }],
            computed_line_boundaries: vec![],
            replacements: vec![],
            scopes: vec![
                SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: base.clone(),
                    used_direction: ParagraphBaseDirection::LeftToRight,
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: Some(base),
                    boundaries: endpoints
                        .then_some(SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let analyzed = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::mark_ahem())
        .unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn finish(p: &PreparedText, mut input: LineSelectionRequest, edges: [f64; 2]) -> FinalizedLine {
    input.controls = input
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
                strut: None,
                boundary: Some(
                    crate::InlineBoundaryAllocation::try_new(
                        if *control.view().kind() == ParagraphControlKind::InlineBoundaryStart {
                            edges[0]
                        } else {
                            edges[1]
                        },
                    )
                    .unwrap(),
                ),
            }),
            _ => None,
        })
        .collect();
    while let Some(occurrence) = p.boundary_context_demand(&input).unwrap() {
        let advance = if *occurrence.view().kind() == ParagraphControlKind::InlineBoundaryStart {
            edges[0]
        } else {
            edges[1]
        };
        input
            .boundary_context
            .push(crate::InlineBoundaryContextAllocation {
                occurrence,
                boundary: crate::InlineBoundaryAllocation::try_new(advance).unwrap(),
            });
    }
    p.select(input)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32, f64, f32, Range)> {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        glyph.advance(),
                        run.origin().x + glyph.pen() + f64::from(glyph.x()),
                        glyph.y(),
                        glyph.range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn premises(p: &PreparedText, anywhere: bool) {
    let analysis = p.analyzed_source();
    assert_eq!(analysis.source().text().code_units(), [65, 769, 66]);
    assert_eq!(analysis.source().id(), Some(Id::from_u64(1_507_000)));
    assert_eq!(analysis.source().revision(), 7);
    assert_eq!(
        analysis.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let origins = p.origin_views().collect::<Vec<_>>();
    assert_eq!(origins.len(), 2);
    for (origin, id, revision, units, aggregate) in [
        (&origins[0], 1_507_001, 11, vec![65], Range::new(0, 1)),
        (&origins[1], 1_507_002, 13, vec![769, 66], Range::new(1, 3)),
    ] {
        assert_eq!(origin.source().id(), Some(Id::from_u64(id)));
        assert_eq!(origin.source().revision(), revision);
        assert_eq!(origin.source().text().code_units(), units);
        assert_eq!(origin.local_range(), Range::new(0, units.len()));
        assert_eq!(origin.aggregate_range(), aggregate);
    }
    let controls = p.control_views().collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    assert_eq!(
        *controls[0].kind(),
        ParagraphControlKind::InlineBoundaryStart
    );
    assert_eq!(*controls[1].kind(), ParagraphControlKind::InlineBoundaryEnd);
    assert_eq!(controls[0].source_position().get(), 0);
    assert_eq!(controls[1].source_position().get(), 1);
    let start = controls[0].handle().captured_control().unwrap();
    let end = controls[1].handle().captured_control().unwrap();
    assert!(start.event_position() < origins[0].capture_position().unwrap());
    assert!(origins[0].capture_position().unwrap() < end.event_position());
    assert!(end.event_position() < origins[1].capture_position().unwrap());
    assert_ne!(start, end);
    assert!(p.control_is_boundary_exact(&controls[0].handle()).unwrap());
    assert!(!p.control_is_boundary_exact(&controls[1].handle()).unwrap());

    let participants = p.participants().collect::<Vec<_>>();
    let at = |source| {
        participants
            .iter()
            .position(|item| matches!(item, PreparedParticipant::Text(unit) if unit.view().source_range() == source))
            .unwrap()
    };
    let base = at(Range::new(0, 1));
    let mark = at(Range::new(1, 2));
    for offer in p.opportunity_views() {
        let cut = offer.boundary().view().participant_offset();
        assert_eq!(base < cut, mark < cut, "no ordinary cut divides A+acute");
    }

    // Qualify the genuine registered font and cross-origin shared shaping
    // before the physical-endpoint assertion. GPOS (400,600)-(100,200) at
    // 10/1000 puts the zero-advance mark at (+3,-4) from A. Both glyphs
    // retain their provider-closed range0..2, independently of origin ranges.
    let control = capture(false, anywhere);
    let line = finish(&control, request(&control), [0.0; 2]);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&line),
        [
            (35, 10.0, 0.0, 0.0, Range::new(0, 2)),
            (100, 0.0, 3.0, -4.0, Range::new(0, 2)),
            (36, 10.0, 10.0, 0.0, Range::new(2, 3)),
        ]
    );
}

fn assert_event_custody(p: &PreparedText) {
    let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
    let units = p.unit_views().map(|v| v.handle()).collect::<Vec<_>>();
    assert_eq!(
        units
            .iter()
            .map(|u| u.view().source_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    assert_eq!(
        p.participants().collect::<Vec<_>>(),
        [
            PreparedParticipant::Control(controls[0].clone()),
            PreparedParticipant::Text(units[0].clone()),
            PreparedParticipant::Control(controls[1].clone()),
            PreparedParticipant::Text(units[1].clone()),
            PreparedParticipant::Text(units[2].clone()),
        ],
        "physical End retains its captured event before the connected mark; exact event custody does not split the shared unit"
    );
}

#[test]
fn break_all_retains_physical_end_event_inside_the_shared_combining_unit() {
    let p = capture(true, false);
    premises(&p, false);
    assert_event_custody(&p);
    let line = finish(&p, request(&p), [0.0; 2]);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&line),
        [
            (35, 10.0, 0.0, 0.0, Range::new(0, 2)),
            (100, 0.0, 3.0, -4.0, Range::new(0, 2)),
            (36, 10.0, 10.0, 0.0, Range::new(2, 3)),
        ]
    );
}

#[test]
fn anywhere_positive_endpoints_preserve_shared_mark_attachment_and_count_each_edge_once() {
    let p = capture(true, true);
    premises(&p, true);
    assert_event_custody(&p);
    // Genuine consumer-used Start2 and End3: A10+mark0 remains one group;
    // both endpoints add5 once, so the following B starts at15, not18.
    let line = finish(&p, request(&p), [2.0, 3.0]);
    assert_eq!(line.feedback().advance, 25.0);
    assert_eq!(line.measurements().fit().advance(), 25.0);
    assert_eq!(
        line.measurements().max_content_contribution().advance(),
        25.0
    );
    assert_eq!(
        glyphs(&line),
        [
            (35, 10.0, 2.0, 0.0, Range::new(0, 2)),
            (100, 0.0, 5.0, -4.0, Range::new(0, 2)),
            (36, 10.0, 15.0, 0.0, Range::new(2, 3)),
        ]
    );
    let slots = line.control_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].inline_bounds(), (0.0, 2.0));
    assert_eq!(slots[1].inline_bounds(), (12.0, 15.0));
    for (slot, control) in slots.iter().zip(p.control_views()) {
        assert_eq!(slot.control(), control.handle());
    }
}

// Flat native ParagraphInput is the existing source-only front door for exact
// endpoint seams. It makes no claim to a captured physical scope pairing.
fn native_pair(text: crate::Text, at: usize, whitespace: WhiteSpace) -> ParagraphInput {
    let source = Source::identified(text, Id::from_u64(1_507_100), 19);
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            line_height: LineHeight::Absolute(10.0),
            white_space: whitespace,
            word_break: WordBreak::BreakAll,
            ..Style::default()
        },
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![ParagraphOrigin {
            text_node: None,
            local: Range::new(0, source.text().len()),
            aggregate: Range::new(0, source.text().len()),
            source,
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![
            ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphControl {
                at,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: (0..2)
            .map(|control| crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::Control(control),
                affinity: Affinity::After,
            })
            .collect(),
    }
}

fn prepare_native(analysis: &AnalyzedSource) -> PreparedText {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::mark_ahem())
        .unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(analysis, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn nested_capture(text: &str) -> PreparedText {
    let input = native_pair(crate::Text::from(text), 1, WhiteSpace::Preserve);
    let origin = |range| ParagraphOrigin {
        text_node: None,
        source: input.source.clone(),
        local: range,
        aggregate: range,
        kind: ParagraphOriginKind::Authored,
    };
    let control = |control| E::Control {
        control,
        affinity: Affinity::After,
    };
    let processed = Arc::new(
        crate::process_contextual_source(ContextualSourceInput {
            scope_alignment_limits: None,
            source: input.source.clone(),
            segment_breaks: input.segment_breaks,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![origin(Range::new(0, 1)), origin(Range::new(1, 3))],
            connections: vec![SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: ParagraphConnectionKind::Continuous,
            }],
            controls: vec![
                ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphControl {
                    at: 1,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
                ParagraphControl {
                    at: 1,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: vec![ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }],
            computed_line_boundaries: vec![],
            replacements: vec![],
            scopes: vec![
                SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: input.base_style,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: Some(SourceProcessingScopeBoundaries { start: 0, end: 3 }),
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: Some(SourceProcessingScopeBoundaries { start: 1, end: 2 }),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                control(0),
                E::EnterScope(2),
                control(1),
                E::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                control(2),
                E::ExitScope(2),
                control(3),
                E::ExitScope(1),
                E::Connection(0),
                E::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    prepare_native(
        &AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
    )
}

#[test]
fn nested_physical_pairs_retain_distinct_coincident_end_events_and_shared_glyphs() {
    let p = nested_capture("A\u{301}B");
    let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
    let units = p.unit_views().map(|v| v.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 4);
    assert_eq!(units.len(), 3);
    let scopes = p
        .captured_paragraph()
        .unwrap()
        .scope_intersections()
        .map(|intersection| intersection.scope())
        .filter(|scope| scope.role() == crate::ScopeRole::Inline)
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), 2);
    for (start, end) in [(0, 3), (1, 2)] {
        let scope = scopes
            .iter()
            .find(|scope| scope.entry_control() == controls[start].captured_control())
            .unwrap();
        assert_eq!(scope.exit_control(), controls[end].captured_control());
    }
    for pair in controls.windows(2) {
        assert_ne!(pair[0], pair[1]);
        assert!(
            pair[0].captured_control().unwrap().event_position()
                < pair[1].captured_control().unwrap().event_position()
        );
    }
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    assert_eq!(
        p.participants().collect::<Vec<_>>(),
        [
            PreparedParticipant::Control(controls[0].clone()),
            PreparedParticipant::Control(controls[1].clone()),
            PreparedParticipant::Text(units[0].clone()),
            PreparedParticipant::Control(controls[2].clone()),
            PreparedParticipant::Control(controls[3].clone()),
            PreparedParticipant::Text(units[1].clone()),
            PreparedParticipant::Text(units[2].clone()),
        ]
    );
    assert!(!p.control_is_boundary_exact(&controls[2]).unwrap());
    assert!(!p.control_is_boundary_exact(&controls[3]).unwrap());
    // Two real Start2/End3 pairs add10, without duplicating a shared glyph.
    let line = finish(&p, request(&p), [2.0, 3.0]);
    assert_eq!(line.feedback().advance, 30.0);
    assert_eq!(
        glyphs(&line),
        [
            (35, 10.0, 4.0, 0.0, Range::new(0, 2)),
            (100, 0.0, 7.0, -4.0, Range::new(0, 2)),
            (36, 10.0, 20.0, 0.0, Range::new(2, 3))
        ]
    );
    assert_eq!(
        line.control_slots()
            .map(|s| s.control())
            .collect::<Vec<_>>(),
        controls
    );
    // The actual captured outer/inner pairs, not a source-offset stack, own
    // these four separate numeric strips around the one shared glyph group.
    for (control, bounds) in
        controls
            .iter()
            .zip([(0.0, 2.0), (2.0, 4.0), (14.0, 17.0), (17.0, 20.0)])
    {
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == *control)
            .unwrap();
        assert_eq!(slot.inline_bounds(), bounds);
    }
}

#[test]
fn native_disconnected_surrogate_atoms_admit_exact_end_but_connected_scalar_refuses() {
    for kind in [
        ParagraphConnectionKind::Disconnected,
        ParagraphConnectionKind::Continuous,
    ] {
        let mut input = native_pair(
            crate::Text::from_code_units(vec![0xd800, 0xdc00]),
            1,
            WhiteSpace::Preserve,
        );
        input.origins = [Range::new(0, 1), Range::new(1, 2)]
            .into_iter()
            .map(|range| ParagraphOrigin {
                text_node: None,
                source: input.source.clone(),
                local: range,
                aggregate: range,
                kind: ParagraphOriginKind::Authored,
            })
            .collect();
        input.connections = vec![crate::ParagraphConnection { at: 1, kind }];
        input.boundary_policies = vec![ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        }];
        let result =
            AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition);
        if kind == ParagraphConnectionKind::Continuous {
            let error = result.unwrap_err();
            assert!(
                matches!(
                    error.detail(),
                    Some(crate::ErrorDetail::InvalidTextProtocol {
                        reason: ProtocolError::InvalidControl { control: 1, at: 1 }
                    })
                ),
                "{error:?}"
            );
        } else {
            let analysis = result.unwrap();
            assert_eq!(analysis.source().text().code_units(), [0xd800, 0xdc00]);
            assert_eq!(
                analysis.graphemes().collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(1, 2)]
            );
            let p = prepare_native(&analysis);
            let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
            let units = p.unit_views().map(|v| v.handle()).collect::<Vec<_>>();
            assert_eq!(
                p.participants().collect::<Vec<_>>(),
                [
                    PreparedParticipant::Control(controls[0].clone()),
                    PreparedParticipant::Text(units[0].clone()),
                    PreparedParticipant::Control(controls[1].clone()),
                    PreparedParticipant::Text(units[1].clone())
                ]
            );
            assert!(p.control_is_boundary_exact(&controls[1]).unwrap());
        }
    }
}

#[test]
fn phase_one_removed_seam_preserves_end_event_but_straddling_replacement_refuses() {
    let analysis = AnalyzedSource::from_paragraph(
        native_pair(crate::Text::from("A  B"), 2, WhiteSpace::Discard),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analysis.source().text().code_units(), [65, 32, 32, 66]);
    assert_eq!(
        analysis.processed_source().processed_text(),
        &crate::Text::from("AB")
    );
    let p = prepare_native(&analysis);
    let end = p.control_views().nth(1).unwrap();
    assert_eq!(end.source_position().get(), 2);
    assert_eq!(end.processed_position().get(), 1);
    let participants = p.participants().collect::<Vec<_>>();
    let end_at = participants
        .iter()
        .position(|p| *p == PreparedParticipant::Control(end.handle()))
        .unwrap();
    let b_at = participants.iter().position(|p| matches!(p, PreparedParticipant::Text(u) if u.view().source_range() == Range::new(3,4))).unwrap();
    assert!(end_at < b_at);
    let line = finish(&p, request(&p), [0.0; 2]);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(line.control_slots().count(), 2);

    // One declared document delimiter replaces the complete source1..3 row.
    // A physical End at2 has no exact emitted scalar side inside that row.
    let mut input = native_pair(crate::Text::from("A\r\nB"), 2, WhiteSpace::PreserveSpaces);
    input.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]);
    let error =
        AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap_err();
    assert!(
        matches!(
            error.detail(),
            Some(crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::InvalidControl { control: 1, at: 2 }
            })
        ),
        "{error:?}"
    );
}

#[test]
fn compatible_case_expansion_keeps_end_between_original_components() {
    let mut input = native_pair(crate::Text::from("ß\u{301}B"), 1, WhiteSpace::Preserve);
    input.base_style.text_transform = crate::TextTransform::Set(
        crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
    );
    let analysis =
        AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert_eq!(analysis.source().text().code_units(), [223, 769, 66]);
    assert_eq!(
        analysis.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let p = prepare_native(&analysis);
    let end = p.control_views().nth(1).unwrap().handle();
    assert_eq!(end.view().source_position().get(), 1);
    assert!(!p.control_is_boundary_exact(&end).unwrap());
    let parts = p.participants().collect::<Vec<_>>();
    let end_at = parts
        .iter()
        .position(|part| *part == PreparedParticipant::Control(end.clone()))
        .unwrap();
    let expanded = parts
        .iter()
        .enumerate()
        .filter_map(|(at, part)| match part {
            PreparedParticipant::Text(unit) if unit.view().source_range() == Range::new(0, 1) => {
                Some(at)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        expanded.len(),
        2,
        "authentic uppercase ß expands into two components"
    );
    assert!(expanded.into_iter().all(|at| at < end_at));
    assert!(parts.iter().enumerate().any(|(at, part)| at > end_at && matches!(part, PreparedParticipant::Text(unit) if unit.view().source_range() == Range::new(1,2))));
}

#[test]
fn transparent_empty_and_out_of_flow_occurrences_do_not_split_the_shared_unit() {
    let mut input = native_pair(crate::Text::from("A\u{301}B"), 1, WhiteSpace::Preserve);
    input.source.inline_box(crate::InlineBox::new(
        Id::from_u64(1_507_199),
        crate::InlineBoxKind::OutOfFlow,
        1,
        crate::Size::new(7.0, 9.0),
    ));
    input.origins = [Range::new(0, 1), Range::new(1, 1), Range::new(1, 3)]
        .into_iter()
        .map(|range| ParagraphOrigin {
            text_node: None,
            source: input.source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    input.connections = (0..2)
        .map(|_| crate::ParagraphConnection {
            at: 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    input.boundary_policies = (0..2)
        .map(|i| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    input.anchor_order = [
        crate::ParagraphAnchorItem::Control(0),
        crate::ParagraphAnchorItem::EmptyOrigin(1),
        crate::ParagraphAnchorItem::Atomic(0),
        crate::ParagraphAnchorItem::Control(1),
    ]
    .into_iter()
    .map(|item| crate::ParagraphAnchor {
        item,
        affinity: Affinity::After,
    })
    .collect();
    // Existing generic After affinity moves Empty/OOF to EGC-end2. Keeping
    // them before physical End1 is a genuine descending projected request;
    // exact endpoint support does not redefine those other roles' affinity.
    let error = AnalyzedSource::from_paragraph(
        input.clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::InvalidAnchorOrder {
                position: 3,
                kind: ProtocolHandleKind::Control,
                occurrence: 1,
            }
        })
    ));
    // A noncrossing, equally authentic co-located request keeps physical End
    // before the transparent occurrences. Their existing After affinity stays.
    input.anchor_order = [
        crate::ParagraphAnchorItem::Control(0),
        crate::ParagraphAnchorItem::Control(1),
        crate::ParagraphAnchorItem::EmptyOrigin(1),
        crate::ParagraphAnchorItem::Atomic(0),
    ]
    .into_iter()
    .map(|item| crate::ParagraphAnchor {
        item,
        affinity: Affinity::After,
    })
    .collect();
    let p = prepare_native(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
    );
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let empty = p.origin_views().nth(1).unwrap();
    assert_eq!(empty.aggregate_range(), Range::new(1, 1));
    assert_eq!(
        p.unit_views()
            .filter(|u| u.empty_origin().as_ref() == Some(&empty.handle()))
            .count(),
        1
    );
    let atomic = p.atomic_views().next().unwrap();
    assert_eq!(atomic.original_box().id(), Id::from_u64(1_507_199));
    assert_eq!(
        atomic.original_box().kind(),
        crate::InlineBoxKind::OutOfFlow
    );
    let end = p.control_views().nth(1).unwrap().handle();
    assert!(!p.control_is_boundary_exact(&end).unwrap());
    let parts = p.participants().collect::<Vec<_>>();
    let at = |source| {
        parts.iter().position(|part| matches!(part, PreparedParticipant::Text(u) if u.view().source_range() == source && u.view().empty_origin().is_none())).unwrap()
    };
    let end_at = parts
        .iter()
        .position(|part| *part == PreparedParticipant::Control(end.clone()))
        .unwrap();
    assert!(at(Range::new(0, 1)) < end_at && end_at < at(Range::new(1, 2)));
    for offer in p.opportunity_views() {
        let cut = offer.boundary().view().participant_offset();
        assert_eq!(at(Range::new(0, 1)) < cut, at(Range::new(1, 2)) < cut);
    }
    let mut input = request(&p);
    input.atomics = vec![AtomicAllocation {
        occurrence: atomic.handle(),
        allocation: AtomicLineAllocation::OutOfFlow,
    }];
    let line = finish(&p, input, [2.0, 3.0]);
    assert_eq!(line.feedback().advance, 25.0);
    assert_eq!(
        glyphs(&line),
        [
            (35, 10.0, 2.0, 0.0, Range::new(0, 2)),
            (100, 0.0, 5.0, -4.0, Range::new(0, 2)),
            (36, 10.0, 15.0, 0.0, Range::new(2, 3))
        ]
    );
    assert_eq!(line.atomic_slots().count(), 1);
    assert_eq!(line.control_slots().count(), 2);
}

#[test]
fn odd_shared_unit_keeps_each_nested_end_interval_on_its_actual_visual_side() {
    use skrifa::{
        GlyphId, MetadataProvider,
        instance::{LocationRef, Size},
        raw::TableProvider,
    };
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    // This is explicitly a .notdef geometry/UBA witness, not Hebrew coverage
    // or a GPOS anchor claim for glyph0. The real retained Ahem glyph0 has a
    // 1000-unit advance at UPEM1000; the actual registered acute maps to100.
    assert!(font.charmap().map(0x05d0_u32).is_none());
    assert_eq!(font.charmap().map(0x0301_u32).unwrap().to_u32(), 100);
    assert_eq!(font.charmap().map(0x42_u32).unwrap().to_u32(), 36);
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(
        font.glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(0)),
        Some(1000.0)
    );
    let p = nested_capture("\u{5d0}\u{301}B");
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [1488, 769, 66]
    );
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let units = p.unit_views().collect::<Vec<_>>();
    assert_eq!(
        units.iter().map(|u| u.source_range()).collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    assert_eq!(
        units.iter().map(|u| u.level().get()).collect::<Vec<_>>(),
        [1, 1, 0]
    );
    let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 4);
    assert!(controls.iter().all(|c| c.view().level().get() == 1));
    for end in &controls[2..] {
        assert!(!p.control_is_boundary_exact(end).unwrap());
    }
    let parts = p.participants().collect::<Vec<_>>();
    let base = parts
        .iter()
        .position(|v| *v == PreparedParticipant::Text(units[0].handle()))
        .unwrap();
    let mark = parts
        .iter()
        .position(|v| *v == PreparedParticipant::Text(units[1].handle()))
        .unwrap();
    for offer in p.opportunity_views() {
        let cut = offer.boundary().view().participant_offset();
        assert_eq!(base < cut, mark < cut);
    }
    // Exact canonical pairs are retained even though their source positions
    // coincide. No array/source-offset pairing convention supplies authority.
    let scopes = p
        .captured_paragraph()
        .unwrap()
        .scope_intersections()
        .map(|i| i.scope())
        .filter(|s| s.role() == crate::ScopeRole::Inline)
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), 2);
    for (start, end) in [(0, 3), (1, 2)] {
        let scope = scopes
            .iter()
            .find(|s| s.entry_control() == controls[start].captured_control())
            .unwrap();
        assert_eq!(scope.exit_control(), controls[end].captured_control());
    }
    let plain = finish(&p, request(&p), [0.0; 2]);
    let plain_glyphs = glyphs(&plain);
    assert_eq!(plain.feedback().advance, 20.0);
    for (id, advance, range) in [
        (0, 10.0, Range::new(0, 2)),
        (100, 0.0, Range::new(0, 2)),
        (36, 10.0, Range::new(2, 3)),
    ] {
        let rows = plain_glyphs
            .iter()
            .filter(|g| g.0 == id)
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, advance);
        assert_eq!(rows[0].4, range);
    }
    let line = finish(&p, request(&p), [2.0, 3.0]);
    assert_eq!(line.feedback().advance, 30.0);
    let used = glyphs(&line);
    assert_eq!(used.len(), plain_glyphs.len());
    for original in &plain_glyphs {
        let actual = used.iter().find(|g| g.0 == original.0).unwrap();
        assert_eq!(
            (actual.1, actual.3, actual.4),
            (original.1, original.3, original.4)
        );
        // On the actual odd group, two End3 strips occupy its physical left;
        // its complete shared glyph packet translates by6 without detaching a
        // mark. The following level0 B sees both End3 and Start2 pairs once.
        assert_eq!(
            actual.2,
            original.2 + if original.0 == 36 { 10.0 } else { 6.0 }
        );
    }
    // L2 reverses the real nested endpoints at odd level: outerEnd,innerEnd,
    // the shared10px glyph packet, innerStart,outerStart, then ordinary B.
    for (control, bounds) in
        controls
            .iter()
            .zip([(18.0, 20.0), (16.0, 18.0), (3.0, 6.0), (0.0, 3.0)])
    {
        let slot = line
            .control_slots()
            .find(|s| s.control() == *control)
            .unwrap();
        assert_eq!(slot.level().get(), 1);
        assert_eq!(slot.inline_bounds(), bounds);
    }
}
