//! Exact expected domain sequences from the authored public capture fixtures.

use super::*;
use CapturedFlowEvent as F;
use SourceProcessingEvent as E;

fn captured(prepared: &PreparedText) -> CapturedParagraph {
    prepared.captured_paragraph().unwrap()
}

fn original(prepared: &PreparedText, index: usize) -> F {
    F::Origin {
        paragraph: captured(prepared),
        position: prepared
            .origin_views()
            .nth(index)
            .unwrap()
            .capture_position()
            .unwrap(),
    }
}

fn atomic(prepared: &PreparedText, index: usize) -> F {
    F::Atomic {
        paragraph: captured(prepared),
        position: prepared
            .atomic_views()
            .nth(index)
            .unwrap()
            .capture_position()
            .unwrap(),
    }
}

fn control(prepared: &PreparedText, index: usize) -> F {
    F::Control(
        prepared
            .control_views()
            .nth(index)
            .unwrap()
            .handle()
            .captured_control()
            .unwrap(),
    )
}

fn flow(domain: &CapturedScope) -> Vec<F> {
    domain.flow_events().unwrap().collect()
}

#[test]
fn plain_block_markers_keep_parent_slots_while_child_contents_stay_separate() {
    for (enabled, known) in [(false, false), (false, true), (true, true)] {
        let prepared = prepare(&continuation(enabled, known));
        let p: Vec<_> = prepared.iter().map(captured).collect();
        assert_eq!(
            flow(&p[0].domain()),
            vec![
                F::ParagraphStart(p[0].clone()),
                control(&prepared[0], 0),
                original(&prepared[0], 0),
                F::ParagraphEnd(p[0].clone()),
                F::BlockStart(p[1].domain()),
                F::BlockEnd(p[1].domain()),
                F::ParagraphStart(p[2].clone()),
                original(&prepared[2], 0),
                F::ParagraphEnd(p[2].clone()),
                F::BlockStart(p[3].domain()),
                F::BlockEnd(p[3].domain()),
                F::ParagraphStart(p[4].clone()),
                control(&prepared[4], 0),
                F::ParagraphEnd(p[4].clone()),
            ]
        );
        for child in [1, 3] {
            assert_eq!(
                flow(&p[child].domain()),
                vec![
                    F::ParagraphStart(p[child].clone()),
                    original(&prepared[child], 0),
                    F::ParagraphEnd(p[child].clone())
                ]
            );
        }
        assert!(inline(&prepared[0]).flow_events().is_none());
    }
}

fn deferred(kind: InlineBoxKind, closed_parent: bool) -> Arc<ProcessedSource> {
    let mut source = Source::new("X");
    for id in [10, 20] {
        source.inline_box(InlineBox::new(
            Id::from_u64(id),
            kind,
            0,
            Size::new(10.0, 10.0),
        ));
    }
    let domain = |atomic| match kind {
        InlineBoxKind::InFlow => SourceProcessingScope::AtomicBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim::default(),
            base_style: style(WordSpaceTransform::None),
            used_direction: ParagraphBaseDirection::LeftToRight,
            atomic,
        },
        InlineBoxKind::OutOfFlow => SourceProcessingScope::OutOfFlowBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim::default(),
            base_style: style(WordSpaceTransform::None),
            used_direction: ParagraphBaseDirection::LeftToRight,
            atomic,
        },
    };
    Arc::new(
        process_contextual_source(ContextualSourceInput {
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![origin(&source, 0, 1)],
            connections: vec![],
            replacements: vec![],
            controls: vec![
                ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: if kind == InlineBoxKind::InFlow {
                (0..2)
                    .flat_map(|atomic| {
                        [
                            ParagraphBoundaryDefinition::AtomicBefore(atomic),
                            ParagraphBoundaryDefinition::AtomicAfter(atomic),
                        ]
                    })
                    .map(|definition| ParagraphBoundaryPolicy {
                        definition,
                        preceding_wrap: Wrap::Word,
                        following_wrap: Wrap::Word,
                        ancestor_wrap: Wrap::Word,
                    })
                    .collect()
            } else {
                vec![]
            },
            scopes: vec![
                block(SourceProcessingTopology::Unknown),
                domain(0),
                domain(1),
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: WhiteSpaceTrim::default(),
                    style: None,
                    boundaries: Some(SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![paragraph(); 3],
            events: {
                let mut events = vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Atomic {
                        atomic: 0,
                        affinity: Affinity::After,
                    },
                    E::Atomic {
                        atomic: 1,
                        affinity: Affinity::After,
                    },
                    E::EnterScope(1),
                    E::EnterParagraph(1),
                    E::EnterScope(3),
                    E::Control {
                        control: 0,
                        affinity: Affinity::After,
                    },
                    E::Control {
                        control: 1,
                        affinity: Affinity::After,
                    },
                    E::ExitScope(3),
                    E::ExitParagraph(1),
                    E::ExitScope(1),
                    E::EnterScope(2),
                    E::EnterParagraph(2),
                    E::Origin {
                        origin: 0,
                        affinity: Affinity::After,
                    },
                    E::ExitParagraph(2),
                    E::ExitScope(2),
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ];
                if closed_parent {
                    let parent_exit = events.remove(events.len() - 2);
                    events.insert(4, parent_exit);
                }
                events
            },
        })
        .unwrap(),
    )
}

#[test]
fn deferred_atomic_and_out_of_flow_contents_have_independent_complete_sequences() {
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        for closed_parent in [false, true] {
            let prepared = prepare(&deferred(kind, closed_parent));
            let p: Vec<_> = prepared.iter().map(captured).collect();
            assert_eq!(
                flow(&p[0].domain()),
                vec![
                    F::ParagraphStart(p[0].clone()),
                    atomic(&prepared[0], 0),
                    atomic(&prepared[0], 1),
                    F::ParagraphEnd(p[0].clone())
                ]
            );
            assert_eq!(
                flow(&p[1].domain()),
                vec![
                    F::ParagraphStart(p[1].clone()),
                    control(&prepared[1], 0),
                    control(&prepared[1], 1),
                    F::ParagraphEnd(p[1].clone())
                ]
            );
            assert_eq!(
                flow(&p[2].domain()),
                vec![
                    F::ParagraphStart(p[2].clone()),
                    original(&prepared[2], 0),
                    F::ParagraphEnd(p[2].clone())
                ]
            );
            let a: Vec<_> = prepared[0]
                .atomic_views()
                .map(|view| view.capture_position().unwrap())
                .collect();
            assert!(a[1] < p[1].domain().enter_position());
            if closed_parent {
                assert!(p[0].exit_position() < p[1].domain().enter_position());
            }
            for child in [1, 2] {
                let expected = match kind {
                    InlineBoxKind::InFlow => CapturedDomainDefinition::AtomicBlock {
                        atomic: a[child - 1].clone(),
                    },
                    InlineBoxKind::OutOfFlow => CapturedDomainDefinition::OutOfFlowBlock {
                        atomic: a[child - 1].clone(),
                    },
                };
                assert_eq!(p[child].domain().domain_definition(), Some(expected));
            }
        }
    }
}

#[test]
fn zero_source_nested_blocks_retain_all_empty_paragraphs_and_parent_markers() {
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: Source::new(""),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: (0..3)
                .map(|_| block(SourceProcessingTopology::Unknown))
                .collect(),
            paragraphs: vec![paragraph(); 4],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::ExitParagraph(0),
                E::EnterScope(1),
                E::EnterParagraph(1),
                E::ExitParagraph(1),
                E::EnterScope(2),
                E::EnterParagraph(2),
                E::ExitParagraph(2),
                E::ExitScope(2),
                E::ExitScope(1),
                E::EnterParagraph(3),
                E::ExitParagraph(3),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let prepared = prepare(&processed);
    let p: Vec<_> = prepared.iter().map(captured).collect();
    assert_eq!(
        flow(&p[0].domain()),
        vec![
            F::ParagraphStart(p[0].clone()),
            F::ParagraphEnd(p[0].clone()),
            F::BlockStart(p[1].domain()),
            F::BlockEnd(p[1].domain()),
            F::ParagraphStart(p[3].clone()),
            F::ParagraphEnd(p[3].clone())
        ]
    );
    assert_eq!(
        flow(&p[1].domain()),
        vec![
            F::ParagraphStart(p[1].clone()),
            F::ParagraphEnd(p[1].clone()),
            F::BlockStart(p[2].domain()),
            F::BlockEnd(p[2].domain())
        ]
    );
    assert_eq!(
        flow(&p[2].domain()),
        vec![
            F::ParagraphStart(p[2].clone()),
            F::ParagraphEnd(p[2].clone())
        ]
    );
    assert!(
        prepared
            .iter()
            .all(|paragraph| paragraph.origin_views().len() == 0
                && paragraph.control_views().len() == 0
                && paragraph.atomic_views().len() == 0
                && paragraph.scopes().len() == 0)
    );
    assert!(
        p.iter()
            .all(|paragraph| paragraph.domain().domain_topology()
                == SourceProcessingTopology::Unknown)
    );
}

#[test]
fn inline_and_connection_facts_do_not_add_flow_events_or_hide_originals() {
    let source = Source::new("AB");
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![origin(&source, 0, 1), origin(&source, 1, 2)],
            connections: vec![SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: ParagraphConnectionKind::Continuous,
            }],
            controls: vec![],
            replacements: vec![],
            boundary_policies: vec![ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }],
            scopes: vec![
                block(SourceProcessingTopology::Unknown),
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: WhiteSpaceTrim::default(),
                    style: None,
                    boundaries: None,
                },
            ],
            paragraphs: vec![paragraph()],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                E::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                E::Connection(0),
                E::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let prepared = prepare(&processed);
    let p = captured(&prepared[0]);
    assert_eq!(
        flow(&p.domain()),
        vec![
            F::ParagraphStart(p.clone()),
            original(&prepared[0], 0),
            original(&prepared[0], 1),
            F::ParagraphEnd(p)
        ]
    );
    assert!(inline(&prepared[0]).flow_events().is_none());
}

#[test]
fn complete_flow_is_capture_owned_across_preparations_and_not_foreign_content_equality() {
    let processed = continuation(false, false);
    let first = prepare(&processed);
    let second = prepare(&processed);
    assert_ne!(first[0], second[0]);
    let root = captured(&first[0]).domain();
    assert_eq!(flow(&root), flow(&captured(&second[0]).domain()));
    let foreign = prepare(&continuation(false, false));
    assert_ne!(flow(&root), flow(&captured(&foreign[0]).domain()));
    // Merely retaining one prepared paragraph cannot shrink the expected flow.
    let first_only = &first[0];
    let expected = flow(&captured(first_only).domain());
    assert!(expected.contains(&F::ParagraphStart(captured(&first[4]))));
    assert!(expected.contains(&F::BlockStart(captured(&first[3]).domain())));
}
