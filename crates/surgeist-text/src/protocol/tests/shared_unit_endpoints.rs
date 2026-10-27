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
                    topology: SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: base.clone(),
                    used_direction: ParagraphBaseDirection::LeftToRight,
                },
                SourceProcessingScope::Inline {
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
    let analyzed =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap()).unwrap();
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
        .finalize(SpacingAllocations::None)
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
