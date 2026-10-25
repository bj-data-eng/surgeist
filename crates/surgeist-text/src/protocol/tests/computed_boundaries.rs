//! Computed box edges are early semantic boundaries with original source custody.
use super::*;
use crate::{
    Affinity, CaseTransform, ComputedLineBoundaryInput, ContextualSourceInput,
    DocumentSegmentBreaks, Id, LineHeight, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    ParagraphConnectionKind, ParagraphOrigin, ParagraphOriginKind, SourceBoxSide,
    SourceProcessingConnection, SourceProcessingEnvelope, SourceProcessingEvent as E,
    SourceProcessingParagraph, SourceProcessingScope, SourceProcessingTopology, TextTransform,
    TextTransformSet, WhiteSpaceTrim, Wrap,
};
use std::sync::Arc;

fn base() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn input(text: &str, ranges: &[Range]) -> ContextualSourceInput {
    let source = Source::identified(text, Id::from_u64(65075), 4);
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    for origin in 0..ranges.len() {
        if origin > 0 {
            events.push(E::Connection(origin - 1));
        }
        events.push(E::Origin {
            origin,
            affinity: Affinity::After,
        });
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    ContextualSourceInput {
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: ranges
            .iter()
            .map(|&range| ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: (1..ranges.len())
            .map(|i| SourceProcessingConnection {
                before_origin: i - 1,
                after_origin: i,
                kind: ParagraphConnectionKind::Continuous,
            })
            .collect(),
        boundary_policies: (1..ranges.len())
            .map(|i| ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(i - 1),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            })
            .collect(),
        controls: vec![],
        computed_line_boundaries: vec![],
        replacements: vec![],
        scopes: vec![SourceProcessingScope::Block {
            topology: SourceProcessingTopology::Complete,
            trim: WhiteSpaceTrim::default(),
            base_style: base(),
            used_direction: ParagraphBaseDirection::LeftToRight,
        }],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events,
    }
}

fn seam(text: &str, at: usize, side: SourceBoxSide) -> ContextualSourceInput {
    let mut value = input(
        text,
        &[
            Range::new(0, at),
            Range::new(at, text.encode_utf16().count()),
        ],
    );
    value.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
    });
    let origin = |origin| E::Origin {
        origin,
        affinity: Affinity::After,
    };
    let middle = match side {
        SourceBoxSide::Before => vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            origin(1),
            E::ExitScope(1),
        ],
        SourceBoxSide::After => vec![
            E::EnterScope(1),
            origin(0),
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ],
    };
    value.events = [
        vec![E::EnterScope(0), E::EnterParagraph(0)],
        middle,
        vec![E::ExitParagraph(0), E::ExitScope(0)],
    ]
    .concat();
    value
}

fn font_system() -> System {
    let mut value = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    value
        .register_fonts(crate::shaping_test_fonts::casing_ahem())
        .unwrap();
    value
}

fn prepare(system: &mut System, processed: &Arc<crate::ProcessedSource>) -> PreparedText {
    let analysis =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap()).unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(&analysis, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn finish(p: &PreparedText) -> Vec<FinalizedLine> {
    let mut start = p.boundary_views().next().unwrap().handle();
    p.opportunity_views()
        .filter(|offer| {
            matches!(
                offer.kind(),
                PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
            )
        })
        .map(|offer| {
            let mut selected = request(p);
            selected.start = start.clone();
            selected.end = offer.boundary();
            let from = selected.start.view().participant_offset();
            let to = selected.end.view().participant_offset();
            selected.participants = p.participants().skip(from).take(to - from).collect();
            selected.controls = selected
                .participants
                .iter()
                .filter_map(|part| {
                    if let PreparedParticipant::Control(control) = part {
                        Some(ControlAllocation {
                            occurrence: control.clone(),
                            strut: None,
                            boundary: matches!(
                                control.view().kind(),
                                crate::ParagraphControlKind::InlineBoundaryStart
                                    | crate::ParagraphControlKind::InlineBoundaryEnd
                            )
                            .then_some(crate::InlineBoundaryAllocation::ZERO),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            selected.context.start = if from == 0 {
                LineStart::Paragraph
            } else {
                LineStart::Continuation
            };
            selected.context.end = if offer.kind() == PreparedBreakKind::Forced {
                LineEnd::Forced
            } else {
                LineEnd::Paragraph
            };
            selected.termination = PreparedTermination::Ordinary(offer.handle());
            while let Some(occurrence) = p.boundary_context_demand(&selected).unwrap() {
                selected
                    .boundary_context
                    .push(crate::InlineBoundaryContextAllocation {
                        occurrence,
                        boundary: crate::InlineBoundaryAllocation::ZERO,
                    });
            }
            let line = p
                .select(selected)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap();
            start = line.selected_line().request().end.clone();
            line
        })
        .collect()
}

fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn placed(p: PreparedText, lines: Vec<FinalizedLine>) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::WholeParagraph,
        lines: lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 10.0 + i as f64 * 10.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })
            .collect(),
    })
    .unwrap()
}

#[test]
fn real_scope_sides_end_lithuanian_casing_context_without_inserting_source_controls() {
    use skrifa::MetadataProvider;
    let bytes = crate::shaping_test_fonts::casing_ahem();
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map(0x49_u32).unwrap().to_u32(), 43);
    assert_eq!(face.charmap().map(0x307_u32).unwrap().to_u32(), 100);
    assert!(face.charmap().map(0x25cc_u32).is_none());
    for side in [SourceBoxSide::Before, SourceBoxSide::After] {
        for forced in [false, true] {
            let mut value = seam("i\u{307}", 1, side);
            let SourceProcessingScope::Block { base_style, .. } = &mut value.scopes[0] else {
                unreachable!()
            };
            base_style.locale = Some("lt".into());
            base_style.text_transform =
                TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Uppercase));
            if forced {
                value
                    .computed_line_boundaries
                    .push(ComputedLineBoundaryInput::Scope { scope: 1, side });
            }
            let source = value.source.clone();
            let processed = Arc::new(crate::process_contextual_source(value).unwrap());
            assert_eq!(processed.processed_text().code_units(), [0x69, 0x307]);
            let p = prepare(&mut font_system(), &processed);
            assert_eq!(p.control_views().count(), 0);
            assert_eq!(p.unit_views().count(), if forced { 2 } else { 1 });
            let lines = finish(&p);
            if forced {
                assert_eq!(
                    p.unit_views().map(|u| u.source_range()).collect::<Vec<_>>(),
                    [Range::new(0, 1), Range::new(1, 2)]
                );
                assert_eq!(lines.len(), 2);
                assert_eq!(glyphs(&lines[0]), [(43, 10.0)]);
                assert_eq!(glyphs(&lines[1]), [(100, 0.0)]);
            } else {
                assert_eq!(lines.len(), 1);
                assert_eq!(glyphs(&lines[0]), [(43, 10.0)]);
            }
            let layout = placed(p, lines);
            assert_eq!(layout.source(), &source);
            assert_eq!(layout.source().text().code_units(), [0x69, 0x307]);
        }
    }
}

#[test]
fn computed_scope_cause_has_its_own_canonical_capture_and_prepared_participant() {
    let mut value = seam("AA", 1, SourceBoxSide::Before);
    value
        .computed_line_boundaries
        .push(ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::Before,
        });
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    let scope = processed.captured_scope(1).unwrap();
    assert_eq!(
        cause.target(),
        crate::CapturedComputedLineBoundaryTarget::Scope {
            scope: scope.clone(),
            side: SourceBoxSide::Before
        }
    );
    assert_eq!(cause.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(cause.paragraph(), processed.captured_paragraph(0).unwrap());
    assert_eq!(cause.source_position().get(), 1);
    let p = prepare(&mut font_system(), &processed);
    assert_eq!(p.control_views().count(), 0);
    let forced = p
        .opportunity_views()
        .filter(|o| o.kind() == PreparedBreakKind::Forced)
        .collect::<Vec<_>>();
    assert_eq!(forced.len(), 1);
    let participant = forced[0].computed_line_boundary().unwrap();
    assert_eq!(participant.view().causes().collect::<Vec<_>>(), [cause]);
    assert_eq!(participant.view().source_position().get(), 1);
    assert_eq!(participant.view().processed_position().get(), 1);
    assert_eq!(p.participants().filter(|part| matches!(part, PreparedParticipant::ComputedLineBoundary(c) if *c == participant)).count(), 1);
    assert!(
        !p.boundary_inside_captured_scope(&forced[0].boundary(), &scope)
            .unwrap()
    );
    let lines = finish(&p);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    assert_eq!(
        placed(p, lines)
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );

    let mut authored = input("AA", &[Range::new(0, 1), Range::new(1, 2)]);
    authored.controls = vec![crate::ParagraphControl {
        at: 1,
        kind: crate::ParagraphControlKind::ForcedBreak,
    }];
    authored.events.insert(
        3,
        E::Control {
            control: 0,
            affinity: Affinity::After,
        },
    );
    let authored = Arc::new(crate::process_contextual_source(authored).unwrap());
    assert!(authored.captured_computed_line_boundary(0).is_none());
    let p = prepare(&mut font_system(), &authored);
    assert_eq!(
        p.control_views().next().unwrap().kind(),
        &crate::ParagraphControlKind::ForcedBreak
    );
    assert!(
        p.opportunity_views()
            .find(|o| o.kind() == PreparedBreakKind::Forced)
            .unwrap()
            .computed_line_boundary()
            .is_none()
    );
    assert_eq!(finish(&p).len(), 2);
}

#[test]
fn early_scope_force_separates_original_base_and_mark_units() {
    for forced in [false, true] {
        let mut value = seam("A\u{307}", 1, SourceBoxSide::Before);
        if forced {
            value
                .computed_line_boundaries
                .push(ComputedLineBoundaryInput::Scope {
                    scope: 1,
                    side: SourceBoxSide::Before,
                });
        }
        let processed = Arc::new(crate::process_contextual_source(value).unwrap());
        let p = prepare(&mut font_system(), &processed);
        let originals = p
            .origin_views()
            .map(|origin| (origin.aggregate_range(), origin.local_range()))
            .collect::<Vec<_>>();
        assert_eq!(
            originals,
            [
                (Range::new(0, 1), Range::new(0, 1)),
                (Range::new(1, 2), Range::new(1, 2))
            ]
        );
        let lines = finish(&p);
        if forced {
            assert_eq!(glyphs(&lines[0]), [(35, 10.0)]);
            assert_eq!(glyphs(&lines[1]), [(100, 0.0)]);
            for (line, original) in lines.iter().zip([Range::new(0, 1), Range::new(1, 2)]) {
                assert_eq!(
                    line.run_views()
                        .flat_map(|run| run.glyphs().map(|glyph| glyph.range()).collect::<Vec<_>>())
                        .collect::<Vec<_>>(),
                    [original]
                );
            }
        } else {
            assert_eq!(glyphs(&lines[0]), [(35, 10.0), (100, 0.0)]);
        }
        let before = p.origin_views().next().unwrap().handle();
        let after = p.origin_views().nth(1).unwrap().handle();
        let layout = placed(p, lines);
        assert_eq!(layout.source().text().code_units(), [0x41, 0x307]);
        let before = layout
            .source_range_fragments(&before, Range::new(0, 1))
            .unwrap();
        let after = layout
            .source_range_fragments(&after, Range::new(1, 2))
            .unwrap();
        assert_eq!(
            before
                .units()
                .next()
                .unwrap()
                .same_unit(&after.units().next().unwrap()),
            !forced
        );
    }
}

#[test]
fn mandatory_scope_edge_has_forced_authority_under_no_wrap_and_word_joiner() {
    let mut value = seam("A\u{2060}A", 1, SourceBoxSide::Before);
    let SourceProcessingScope::Block { base_style, .. } = &mut value.scopes[0] else {
        unreachable!()
    };
    base_style.wrap = Wrap::None;
    for policy in &mut value.boundary_policies {
        policy.preceding_wrap = Wrap::None;
        policy.following_wrap = Wrap::None;
        policy.ancestor_wrap = Wrap::None;
    }
    value
        .computed_line_boundaries
        .push(ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::Before,
        });
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let p = prepare(&mut font_system(), &processed);
    assert_eq!(
        p.opportunity_views().map(|o| o.kind()).collect::<Vec<_>>(),
        [PreparedBreakKind::Forced, PreparedBreakKind::ParagraphEnd]
    );
    let error = p.select(request(&p)).unwrap_err();
    assert!(matches!(
        reason(&error),
        ProtocolError::IllegalOpportunity { .. }
    ));
    let lines = finish(&p);
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[0].selected_line().request().context.end,
        LineEnd::Forced
    );
    let layout = placed(p, lines);
    assert_eq!(layout.source().text().code_units(), [0x41, 0x2060, 0x41]);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 3)]
    );
}

fn paired() -> ContextualSourceInput {
    let mut value = seam("AA", 1, SourceBoxSide::Before);
    value.controls = vec![
        crate::ParagraphControl {
            at: 1,
            kind: crate::ParagraphControlKind::InlineBoundaryStart,
        },
        crate::ParagraphControl {
            at: 2,
            kind: crate::ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    let SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(crate::SourceProcessingScopeBoundaries { start: 0, end: 1 });
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Connection(0),
        E::EnterScope(1),
        E::Control {
            control: 0,
            affinity: Affinity::Before,
        },
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::Control {
            control: 1,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    value
}

#[test]
fn a_real_endpoint_force_does_not_replace_authored_control_identity() {
    let mut value = paired();
    value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Endpoint { control: 0 }];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let captured = processed.captured_control(0).unwrap();
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(
        cause.target(),
        crate::CapturedComputedLineBoundaryTarget::Endpoint(captured.clone())
    );
    let p = prepare(&mut font_system(), &processed);
    assert_eq!(
        p.control_views()
            .map(|v| v.kind().clone())
            .collect::<Vec<_>>(),
        [
            crate::ParagraphControlKind::InlineBoundaryStart,
            crate::ParagraphControlKind::InlineBoundaryEnd
        ]
    );
    let start = p
        .control_views()
        .find(|v| v.handle().captured_control().as_ref() == Some(&captured))
        .unwrap()
        .handle();
    assert!(p.control_is_boundary_exact(&start).unwrap());
    let force = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::Forced)
        .unwrap();
    assert_eq!(
        force
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [cause]
    );
    assert!(
        !p.boundary_consumes_control(&force.boundary(), &start)
            .unwrap()
    );
    let lines = finish(&p);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    assert_eq!(placed(p, lines).source().text().code_units(), [0x41, 0x41]);
}

#[test]
fn a_computed_atomic_edge_retains_actual_occurrence_and_distinct_side() {
    for side in [SourceBoxSide::Before, SourceBoxSide::After] {
        let mut value = input("AA", &[Range::new(0, 1), Range::new(1, 2)]);
        value.source.inline_box(crate::InlineBox::new(
            Id::from_u64(7501),
            crate::InlineBoxKind::InFlow,
            1,
            crate::Size::new(10.0, 10.0),
        ));
        value.boundary_policies.extend(
            [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }),
        );
        value.events.insert(
            3,
            E::Atomic {
                atomic: 0,
                affinity: Affinity::After,
            },
        );
        value.computed_line_boundaries =
            vec![ComputedLineBoundaryInput::Atomic { atomic: 0, side }];
        let processed = Arc::new(crate::process_contextual_source(value).unwrap());
        let cause = processed.captured_computed_line_boundary(0).unwrap();
        assert_eq!(
            cause.target(),
            crate::CapturedComputedLineBoundaryTarget::Atomic {
                position: processed.captured_atomic(0).unwrap(),
                side
            }
        );
        assert_eq!(cause.source_position().get(), 1);
        let p = prepare(&mut font_system(), &processed);
        let force = p
            .opportunity_views()
            .find(|v| v.kind() == PreparedBreakKind::Forced)
            .unwrap();
        assert_eq!(
            force
                .computed_line_boundary()
                .unwrap()
                .view()
                .causes()
                .collect::<Vec<_>>(),
            [cause]
        );
        let before = p
            .participants()
            .take(force.boundary().view().participant_offset())
            .filter(|part| matches!(part, PreparedParticipant::Atomic(_)))
            .count();
        assert_eq!(before, usize::from(side == SourceBoxSide::After));
        assert_eq!(p.atomic_views().count(), 1);
        assert_eq!(p.control_views().count(), 0);
    }
}

#[test]
fn changed_causes_do_not_contaminate_retained_source_or_preparations() {
    let clear = seam("AA", 1, SourceBoxSide::Before);
    let mut forced = clear.clone();
    forced.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }];
    let original = clear.source.clone();
    let no_force = Arc::new(crate::process_contextual_source(clear).unwrap());
    let with_force = Arc::new(crate::process_contextual_source(forced.clone()).unwrap());
    let independent_capture = Arc::new(crate::process_contextual_source(forced).unwrap());
    let mut system = font_system();
    let old = prepare(&mut system, &no_force);
    let current = prepare(&mut system, &with_force);
    let repeated = prepare(&mut system, &with_force);
    let foreign = prepare(&mut system, &independent_capture);
    let old_layout = placed(old.clone(), finish(&old));
    let new_layout = placed(current.clone(), finish(&current));
    assert_eq!(old_layout.source(), &original);
    assert_eq!(new_layout.source(), &original);
    assert_eq!(old_layout.lines().len(), 1);
    assert_eq!(old_layout.lines()[0].range(), Range::new(0, 2));
    assert_eq!(
        new_layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_ne!(old_layout.key(), new_layout.key());
    assert_eq!(
        finish(&repeated)
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    assert_eq!(
        with_force.captured_computed_line_boundary(0),
        repeated
            .opportunity_views()
            .find(|o| o.kind() == PreparedBreakKind::Forced)
            .unwrap()
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .next()
    );
    assert_ne!(
        with_force.captured_computed_line_boundary(0),
        independent_capture.captured_computed_line_boundary(0)
    );
    let mut mixed = request(&current);
    mixed.termination = PreparedTermination::Ordinary(
        foreign
            .opportunity_views()
            .find(|o| o.kind() == PreparedBreakKind::Forced)
            .unwrap()
            .handle(),
    );
    assert!(matches!(
        reason(&current.select(mixed).unwrap_err()),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::Opportunity,
            ..
        }
    ));
    assert_eq!(glyphs(&finish(&old)[0]), [(35, 10.0), (35, 10.0)]);
    assert_eq!(old_layout.lines()[0].range(), Range::new(0, 2));
}

#[test]
fn computed_hard_boundary_resets_an_authentic_embedding_before_receiving_text() {
    for forced in [false, true] {
        let mut value = seam("AAA", 1, SourceBoxSide::Before);
        value.controls = vec![crate::ParagraphControl {
            at: 1,
            kind: crate::ParagraphControlKind::RightToLeftOverride,
        }];
        let enter = value
            .events
            .iter()
            .position(|e| matches!(e, E::EnterScope(1)))
            .unwrap();
        value.events.insert(
            enter,
            E::Control {
                control: 0,
                affinity: Affinity::After,
            },
        );
        if forced {
            value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: SourceBoxSide::Before,
            }];
        }
        let processed = Arc::new(crate::process_contextual_source(value).unwrap());
        let p = prepare(&mut font_system(), &processed);
        // UAX9 P1/X8 ends an explicit override at a paragraph separator. The
        // computed owner is semantic LF; it does not become authored text.
        assert_eq!(
            p.unit_views().map(|u| u.level().get()).collect::<Vec<_>>(),
            if forced { vec![0, 0, 0] } else { vec![0, 1, 1] }
        );
        assert_eq!(
            p.control_views()
                .map(|c| c.kind().clone())
                .collect::<Vec<_>>(),
            [crate::ParagraphControlKind::RightToLeftOverride]
        );
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [0x41, 0x41, 0x41]
        );
        assert_eq!(finish(&p).len(), if forced { 2 } else { 1 });
    }
}

#[test]
fn distinct_sides_of_a_real_empty_scope_keep_equal_offset_mandatory_progress() {
    let mut value = input("AA", &[Range::new(0, 1), Range::new(1, 2)]);
    value.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
    });
    value
        .events
        .splice(3..3, [E::EnterScope(1), E::ExitScope(1)]);
    value.computed_line_boundaries = vec![
        ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::Before,
        },
        ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::After,
        },
    ];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let before = processed.captured_computed_line_boundary(0).unwrap();
    let after = processed.captured_computed_line_boundary(1).unwrap();
    assert_ne!(before, after);
    assert_eq!(
        (
            before.source_position().get(),
            after.source_position().get()
        ),
        (1, 1)
    );
    let p = prepare(&mut font_system(), &processed);
    let forces = p
        .opportunity_views()
        .filter(|o| o.kind() == PreparedBreakKind::Forced)
        .map(|o| o.handle())
        .collect::<Vec<_>>();
    assert_eq!(forces.len(), 2);
    assert_ne!(forces[0].view().boundary(), forces[1].view().boundary());
    assert_eq!(
        forces[0]
            .view()
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [before]
    );
    assert_eq!(
        forces[1]
            .view()
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [after]
    );
    let lines = finish(&p);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 0.0, 10.0]
    );
    assert_eq!(
        placed(p, lines)
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 1), Range::new(1, 2)]
    );
}

#[test]
fn real_exterior_scope_edges_keep_leading_and_terminal_empty_groups() {
    let mut value = input("A", &[Range::new(0, 1)]);
    value.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
    });
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    value.computed_line_boundaries = vec![
        ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::Before,
        },
        ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::After,
        },
    ];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let p = prepare(&mut font_system(), &processed);
    assert_eq!(
        p.opportunity_views().map(|o| o.kind()).collect::<Vec<_>>(),
        [
            PreparedBreakKind::Forced,
            PreparedBreakKind::Forced,
            PreparedBreakKind::ParagraphEnd
        ]
    );
    let lines = finish(&p);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [0.0, 10.0, 0.0]
    );
    assert_eq!(
        placed(p, lines)
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 0), Range::new(0, 1), Range::new(1, 1)]
    );
}

fn refusal(value: ContextualSourceInput, expected: crate::ComputedLineBoundaryError) {
    let error = crate::process_contextual_source(value).unwrap_err();
    assert!(
        matches!(error.detail(),Some(crate::ErrorDetail::SourceProcessing { reason:crate::SourceProcessingError::ComputedLineBoundary { request:0,reason } }) if *reason==expected),
        "{error:?}"
    );
}

#[test]
fn computed_targets_refuse_bounds_wrong_roles_and_uncaptured_paragraph_edges() {
    for target in [
        ComputedLineBoundaryInput::Scope {
            scope: 99,
            side: SourceBoxSide::Before,
        },
        ComputedLineBoundaryInput::Endpoint { control: 99 },
        ComputedLineBoundaryInput::Atomic {
            atomic: 99,
            side: SourceBoxSide::After,
        },
    ] {
        let mut value = input("A", &[Range::new(0, 1)]);
        value.computed_line_boundaries = vec![target];
        refusal(value, crate::ComputedLineBoundaryError::TargetBounds);
    }
    let mut block = input("A", &[Range::new(0, 1)]);
    block.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 0,
        side: SourceBoxSide::Before,
    }];
    refusal(block, crate::ComputedLineBoundaryError::TargetRole);
    let mut control = input("AA", &[Range::new(0, 1), Range::new(1, 2)]);
    control.controls = vec![crate::ParagraphControl {
        at: 1,
        kind: crate::ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(base()),
        },
    }];
    control.events.insert(
        3,
        E::Control {
            control: 0,
            affinity: Affinity::After,
        },
    );
    control.computed_line_boundaries = vec![ComputedLineBoundaryInput::Endpoint { control: 0 }];
    refusal(control, crate::ComputedLineBoundaryError::TargetRole);
    let mut outside = input("A", &[Range::new(0, 1)]);
    outside.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
    });
    outside
        .events
        .splice(1..1, [E::EnterScope(1), E::ExitScope(1)]);
    outside.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }];
    refusal(outside, crate::ComputedLineBoundaryError::MissingParagraph);
}

#[test]
fn computed_edges_refuse_surrogate_and_selected_document_delimiter_interiors() {
    let mut scalar = seam("\u{1f600}", 1, SourceBoxSide::Before);
    scalar.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }];
    refusal(scalar, crate::ComputedLineBoundaryError::InsideScalar);
    let mut whole = seam("\u{1f600}A", 2, SourceBoxSide::Before);
    whole.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }];
    let accepted = crate::process_contextual_source(whole).unwrap();
    assert_eq!(
        accepted
            .captured_computed_line_boundary(0)
            .unwrap()
            .source_position()
            .get(),
        2
    );
    assert_eq!(
        accepted.processed_text().code_units(),
        [0xd83d, 0xde00, 0x41]
    );
    let mut delimiter = seam("A\r\nA", 2, SourceBoxSide::Before);
    delimiter.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]);
    delimiter.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }];
    refusal(delimiter, crate::ComputedLineBoundaryError::InsideDelimiter);
    let mut fallback = seam("A\r\nA", 2, SourceBoxSide::Before);
    fallback.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }];
    let accepted = crate::process_contextual_source(fallback).unwrap();
    assert_eq!(
        accepted
            .captured_computed_line_boundary(0)
            .unwrap()
            .source_position()
            .get(),
        2
    );
    // Under LineFeedDefault only LF is a document delimiter; preserved CR is
    // ordinary whitespace normalized to SPACE, not the first half of CRLF.
    assert_eq!(
        accepted.processed_text().code_units(),
        [0x41, 0x20, 0x0a, 0x41]
    );
}

#[test]
fn authenticated_same_edge_retains_distinct_causes_and_idempotent_literal_duplicates() {
    let mut value = paired();
    let scope = ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    };
    value.computed_line_boundaries = vec![
        scope,
        ComputedLineBoundaryInput::Endpoint { control: 0 },
        scope,
    ];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let scoped = processed.captured_computed_line_boundary(0).unwrap();
    let endpoint = processed.captured_computed_line_boundary(1).unwrap();
    assert_eq!(
        processed.captured_computed_line_boundary(2).unwrap(),
        scoped
    );
    assert_ne!(scoped, endpoint);
    let p = prepare(&mut font_system(), &processed);
    let forces = p
        .opportunity_views()
        .filter(|o| o.kind() == PreparedBreakKind::Forced)
        .collect::<Vec<_>>();
    assert_eq!(forces.len(), 1);
    assert_eq!(
        forces[0]
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [scoped, endpoint]
    );
    assert_eq!(
        finish(&p)
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    assert_eq!(p.control_views().count(), 2);
}

#[test]
fn out_of_flow_atomic_is_not_an_outer_edge_but_its_own_inline_domain_can_force() {
    let mut value = input(
        "AAA",
        &[Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)],
    );
    value.source.inline_box(crate::InlineBox::new(
        Id::from_u64(7502),
        crate::InlineBoxKind::OutOfFlow,
        1,
        crate::Size::new(10.0, 10.0),
    ));
    value.scopes.push(SourceProcessingScope::OutOfFlowBlock {
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: base(),
        used_direction: ParagraphBaseDirection::LeftToRight,
        atomic: 0,
    });
    value.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
    });
    value.paragraphs.push(SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    });
    value.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 2,
        kind: ParagraphConnectionKind::Continuous,
    }];
    value.boundary_policies.truncate(1);
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Atomic {
            atomic: 0,
            affinity: Affinity::After,
        },
        E::EnterScope(1),
        E::EnterParagraph(1),
        E::EnterScope(2),
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::ExitScope(2),
        E::ExitParagraph(1),
        E::ExitScope(1),
        E::Connection(0),
        E::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let mut invalid = value.clone();
    invalid.computed_line_boundaries = vec![ComputedLineBoundaryInput::Atomic {
        atomic: 0,
        side: SourceBoxSide::Before,
    }];
    refusal(invalid, crate::ComputedLineBoundaryError::TargetRole);
    value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 2,
        side: SourceBoxSide::Before,
    }];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(cause.domain(), processed.captured_scope(1).unwrap());
    assert_eq!(cause.paragraph(), processed.captured_paragraph(1).unwrap());
    let mut system = font_system();
    let outer = prepare(&mut system, &processed);
    assert!(
        !outer
            .opportunity_views()
            .any(|o| o.kind() == PreparedBreakKind::Forced)
    );
    let child_analysis =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().nth(1).unwrap()).unwrap();
    let child = system
        .prepare(
            &crate::test_wrapping::bind_missing(&child_analysis, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap();
    assert_eq!(
        child
            .opportunity_views()
            .filter(|o| o.kind() == PreparedBreakKind::Forced)
            .count(),
        1
    );
    assert_eq!(
        finish(&child)
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [0.0, 10.0]
    );
}

#[test]
fn a_removed_only_scope_edge_retains_original_cause_and_translated_processed_position() {
    let mut value = seam(" A", 1, SourceBoxSide::After);
    let SourceProcessingScope::Block { base_style, .. } = &mut value.scopes[0] else {
        unreachable!()
    };
    base_style.white_space = WhiteSpace::Discard;
    value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::After,
    }];
    let source = value.source.clone();
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    assert_eq!(processed.processed_text().code_units(), [0x41]);
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(cause.source_position().get(), 1);
    let p = prepare(&mut font_system(), &processed);
    assert_eq!(
        p.origin_views()
            .map(|o| o.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let force = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .computed_line_boundary()
        .unwrap();
    assert_eq!(force.view().source_position().get(), 1);
    assert_eq!(force.view().processed_position().get(), 0);
    assert_eq!(force.view().causes().collect::<Vec<_>>(), [cause]);
    let lines = finish(&p);
    assert!(glyphs(&lines[0]).is_empty());
    assert_eq!(glyphs(&lines[1]), [(35, 10.0)]);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [0.0, 10.0]
    );
    assert_eq!(placed(p, lines).source(), &source);
}

#[test]
fn computed_forced_edge_stops_opposite_endpoint_context_demand() {
    let mut value = paired();
    value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Endpoint { control: 0 }];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let p = prepare(&mut font_system(), &processed);
    let force = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let mut selected = request(&p);
    selected.end = force.view().boundary();
    selected.participants = p
        .participants()
        .take(selected.end.view().participant_offset())
        .collect();
    selected.context.end = LineEnd::Forced;
    selected.termination = PreparedTermination::Ordinary(force);
    // A mandatory boundary terminates retained shaping context as well as the
    // selected body; the receiving scope's endpoint belongs beyond that stop.
    assert!(p.boundary_context_demand(&selected).unwrap().is_none());
    let line = p
        .select(selected)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(glyphs(&line), [(35, 10.0)]);
}

#[test]
fn computed_forced_receiving_body_stops_preceding_endpoint_context_demand() {
    let mut value = paired();
    value.controls[0].at = 0;
    value.controls[1].at = 1;
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::Control {
            control: 0,
            affinity: Affinity::Before,
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
    value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Endpoint { control: 1 }];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let p = prepare(&mut font_system(), &processed);
    let force = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let mut selected = request(&p);
    selected.start = force.view().boundary();
    selected.participants = p
        .participants()
        .skip(selected.start.view().participant_offset())
        .collect();
    selected.context.start = LineStart::Continuation;
    assert!(p.boundary_context_demand(&selected).unwrap().is_none());
    let line = p
        .select(selected)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(glyphs(&line), [(35, 10.0)]);
}

#[test]
fn computed_forced_terminal_offer_has_cause_but_coincident_paragraph_end_does_not() {
    let mut value = input("A", &[Range::new(0, 1)]);
    value.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
    });
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::After,
    }];
    let processed = Arc::new(crate::process_contextual_source(value).unwrap());
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    let p = prepare(&mut font_system(), &processed);
    let force = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Forced)
        .unwrap();
    let end = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap();
    assert_eq!(force.boundary(), end.boundary());
    assert_eq!(
        force
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [cause]
    );
    assert!(end.computed_line_boundary().is_none());
    assert_eq!(
        finish(&p)
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 0.0]
    );
}

#[test]
fn foreign_computed_participant_is_rejected_even_when_capture_and_anchor_match() {
    let mut value = seam("AA", 1, SourceBoxSide::Before);
    value.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }];
    let processed = Arc::new(crate::process_contextual_source(value.clone()).unwrap());
    let other_capture = Arc::new(crate::process_contextual_source(value).unwrap());
    let mut system = font_system();
    let local = prepare(&mut system, &processed);
    let same_capture = prepare(&mut system, &processed);
    let foreign_capture = prepare(&mut system, &other_capture);
    let force = local
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let mut selected = request(&local);
    selected.end = force.view().boundary();
    selected.participants = local
        .participants()
        .take(selected.end.view().participant_offset())
        .collect();
    selected.context.end = LineEnd::Forced;
    selected.termination = PreparedTermination::Ordinary(force);
    assert_eq!(
        local
            .select(selected.clone())
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        10.0
    );
    assert_eq!(
        processed.captured_computed_line_boundary(0),
        same_capture
            .opportunity_views()
            .find(|offer| offer.kind() == PreparedBreakKind::Forced)
            .unwrap()
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .next()
    );
    assert_ne!(
        processed.captured_computed_line_boundary(0),
        other_capture.captured_computed_line_boundary(0)
    );
    for other in [same_capture, foreign_capture] {
        let foreign = other
            .opportunity_views()
            .find(|offer| offer.kind() == PreparedBreakKind::Forced)
            .unwrap()
            .computed_line_boundary()
            .unwrap();
        assert_eq!(foreign.view().source_position().get(), 1);
        let mut mixed = selected.clone();
        let slot = mixed
            .participants
            .iter_mut()
            .find(|part| matches!(part, PreparedParticipant::ComputedLineBoundary(_)))
            .unwrap();
        *slot = PreparedParticipant::ComputedLineBoundary(foreign);
        assert!(matches!(
            reason(&local.select(mixed).unwrap_err()),
            ProtocolError::ForeignPreparation {
                kind: ProtocolHandleKind::ComputedLineBoundary,
                ..
            }
        ));
    }
}

#[test]
fn later_expanded_wbr_preserves_an_earlier_computed_hard_base_mark_boundary() {
    use crate::WordSpaceTransform;
    use skrifa::MetadataProvider;

    let bytes = crate::shaping_test_fonts::casing_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map(0x41_u32).unwrap().to_u32(), 35);
    assert_eq!(font.charmap().map(0x307_u32).unwrap().to_u32(), 100);
    assert_eq!(font.charmap().map(0x20_u32).unwrap().to_u32(), 3);
    for (forced, operation) in [
        (false, WordSpaceTransform::Space),
        (true, WordSpaceTransform::None),
        (true, WordSpaceTransform::Space),
    ] {
        let mut value = input(
            "A\u{307}AA",
            &[
                Range::new(0, 1),
                Range::new(1, 2),
                Range::new(2, 3),
                Range::new(3, 4),
            ],
        );
        value.scopes.push(SourceProcessingScope::Inline {
            trim: WhiteSpaceTrim::default(),
            style: None,
            boundaries: None,
        });
        value.controls = vec![crate::ParagraphControl {
            at: 3,
            kind: crate::ParagraphControlKind::Wbr {
                style: Box::new(Style {
                    word_space_transform: operation,
                    ..base()
                }),
            },
        }];
        value.computed_line_boundaries = if forced {
            vec![ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: SourceBoxSide::Before,
            }]
        } else {
            Vec::new()
        };
        // The original mark is inside its genuine captured inline. The later
        // Wbr is outside it, between two separate original A occurrences.
        value.events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            E::Connection(0),
            E::EnterScope(1),
            E::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            E::ExitScope(1),
            E::Connection(1),
            E::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            E::Control {
                control: 0,
                affinity: Affinity::After,
            },
            E::Connection(2),
            E::Origin {
                origin: 3,
                affinity: Affinity::After,
            },
            E::ExitParagraph(0),
            E::ExitScope(0),
        ];
        let source = value.source.clone();
        let processed = Arc::new(crate::process_contextual_source(value).unwrap());
        assert_eq!(
            processed.processed_text().code_units(),
            [0x41, 0x307, 0x41, 0x41]
        );
        let analysis =
            AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
                .unwrap_or_else(|error| {
                    panic!("forced={forced}, operation={operation:?}: {error:?}")
                });
        let p = font_system()
            .prepare(
                &crate::test_wrapping::bind_missing(&analysis, Wrap::Word),
                PreparationOptions::default(),
            )
            .unwrap();
        let wbr = p.control_views().next().unwrap().handle();
        assert_eq!(p.control_views().count(), 1);
        assert_eq!(wbr.captured_control(), processed.captured_control(0));
        assert_eq!(wbr.view().source_position().get(), 3);
        let wbr_view = wbr.view();
        let crate::ParagraphControlKind::Wbr { style } = wbr_view.kind() else {
            unreachable!()
        };
        assert_eq!(style.word_space_transform, operation);
        let generated = p
            .unit_views()
            .filter(|unit| unit.controls().any(|control| control == wbr))
            .collect::<Vec<_>>();
        if operation == WordSpaceTransform::Space {
            assert_eq!(generated.len(), 1);
            assert!(generated[0].origins().next().is_none());
            assert_eq!(generated[0].source_range(), Range::new(3, 3));
            assert!(generated[0].processed_range().is_empty());
            assert_eq!(
                generated[0].controls().collect::<Vec<_>>().as_slice(),
                std::slice::from_ref(&wbr)
            );
        } else {
            assert!(generated.is_empty());
        }
        let origins = p
            .origin_views()
            .map(|origin| origin.handle())
            .collect::<Vec<_>>();
        assert_eq!(
            origins
                .iter()
                .map(|origin| (origin.view().aggregate_range(), origin.view().local_range()))
                .collect::<Vec<_>>(),
            [
                (Range::new(0, 1), Range::new(0, 1)),
                (Range::new(1, 2), Range::new(1, 2)),
                (Range::new(2, 3), Range::new(2, 3)),
                (Range::new(3, 4), Range::new(3, 4)),
            ]
        );
        if operation == WordSpaceTransform::Space {
            // Qualify the later emission on its own actual issued body range
            // before examining the earlier hard-edge TCU custody.
            let tail_start = p
                .participants()
                .position(|part| match part {
                    PreparedParticipant::Text(unit) => unit
                        .view()
                        .origins()
                        .any(|piece| piece.origin() == origins[2]),
                    _ => false,
                })
                .unwrap();
            let mut tail = request(&p);
            tail.start = p
                .boundary_views()
                .find(|boundary| boundary.participant_offset() == tail_start)
                .unwrap()
                .handle();
            tail.participants = p.participants().skip(tail_start).collect();
            assert!(
                tail.participants
                    .iter()
                    .all(|part| matches!(part, PreparedParticipant::Text(_)))
            );
            tail.context.start = LineStart::Continuation;
            let tail = p
                .select(tail)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap();
            assert_eq!(
                tail.mapping_views()
                    .filter(|mapping| mapping.kind() == PresentationMappingKind::GeneratedSeparator)
                    .map(|mapping| (
                        mapping.separator_control(),
                        mapping.source_range(),
                        mapping.processed_range().is_empty()
                    ))
                    .collect::<Vec<_>>(),
                [(Some(wbr.clone()), Range::new(3, 3), true)]
            );
            assert!(
                tail.run_views()
                    .any(|run| run.glyphs().any(|glyph| glyph.id() == 3))
            );
        }
        if !forced {
            continue;
        }
        // This is original occurrence/TCU custody, not a mark glyph-count or
        // advance assumption. Later generated text cannot reconnect a hard edge.
        for (origin, expected) in origins
            .iter()
            .take(2)
            .zip([Range::new(0, 1), Range::new(1, 2)])
        {
            let units = p
                .unit_views()
                .filter(|unit| unit.origins().any(|piece| piece.origin() == *origin))
                .map(|unit| unit.source_range())
                .collect::<Vec<_>>();
            assert!(!units.is_empty());
            assert!(
                units.iter().all(|range| *range == expected),
                "{operation:?}: origin {expected:?} belongs to {units:?}"
            );
        }
        let lines = finish(&p);
        assert_eq!(lines.len(), 2);
        let mappings = lines
            .iter()
            .flat_map(|line| {
                line.mapping_views()
                    .filter(|mapping| mapping.kind() == PresentationMappingKind::GeneratedSeparator)
                    .map(|mapping| {
                        (
                            mapping.separator_control(),
                            mapping.source_range(),
                            mapping.processed_range().is_empty(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        if operation == WordSpaceTransform::Space {
            assert_eq!(mappings, [(Some(wbr), Range::new(3, 3), true)]);
            assert!(
                lines[1]
                    .run_views()
                    .any(|run| run.glyphs().any(|glyph| glyph.id() == 3))
            );
        } else {
            assert!(mappings.is_empty());
        }
        let layout = placed(p, lines);
        assert_eq!(layout.source(), &source);
        assert_eq!(
            layout.source().text().code_units(),
            [0x41, 0x307, 0x41, 0x41]
        );
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 4)]
        );
        let before = layout
            .source_range_fragments(&origins[0], Range::new(0, 1))
            .unwrap();
        let after = layout
            .source_range_fragments(&origins[1], Range::new(1, 2))
            .unwrap();
        assert!(
            !before
                .units()
                .next()
                .unwrap()
                .same_unit(&after.units().next().unwrap())
        );
    }
}
