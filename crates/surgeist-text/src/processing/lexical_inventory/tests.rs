//! Functional qualification of source-owned occurrences before mapping
//! coalescence. No font, shaped glyph, or later cell-selection result is used.

use super::*;
use crate::processing::{MappingKind, identity};
use crate::*;
use std::sync::Arc;

type CharacterFact = (
    usize,
    Range,
    WhiteSpace,
    LexicalCharacterRole,
    Range,
    Option<DeletionCause>,
);

fn records(processed: &ProcessedSource) -> (&LexicalContext, &[LexicalRecord]) {
    let LexicalState::Executed { context, records } = &processed.lexical else {
        panic!("this public processing operation executed Phase I")
    };
    (context, records)
}

fn characters(processed: &ProcessedSource) -> Vec<CharacterFact> {
    records(processed)
        .1
        .iter()
        .filter_map(|record| {
            let LexicalRecordKind::Character {
                source,
                mode,
                role,
                disposition,
            } = record.kind
            else {
                return None;
            };
            let (processed, cause) = match disposition {
                LexicalDisposition::Emitted { processed } => (processed, None),
                LexicalDisposition::Deleted { processed, cause } => {
                    (Range::new(processed, processed), Some(cause))
                }
            };
            Some((record.domain, source, mode, role, processed, cause))
        })
        .collect()
}

fn plaintext<'a>(items: impl Iterator<Item = PlaintextItem<'a>>) -> Vec<u16> {
    items
        .flat_map(|item| match item {
            PlaintextItem::Text(text) => text.code_units().to_vec(),
            PlaintextItem::InlineBox(_) => vec![],
        })
        .collect()
}

fn ordinary(processed: &ProcessedSource, source: &Source, expected: &str) {
    assert_eq!(processed.source(), source);
    assert_eq!(processed.processed_text(), &Text::from(expected));
    assert_eq!(
        plaintext(processed.plaintext_items(Affinity::After)),
        expected.encode_utf16().collect::<Vec<_>>()
    );
}

fn local(text: &str, mode: WhiteSpace, breaks: DocumentSegmentBreaks) -> ProcessedSource {
    let source = Source::new(text);
    let processed = process_source_whitespace(
        source.clone(),
        Style {
            white_space: mode,
            ..Style::default()
        },
        breaks,
    )
    .unwrap();
    assert_eq!(processed.source(), &source);
    assert!(matches!(records(&processed).0, LexicalContext::SourceLocal));
    processed
}

fn origin(source: &Source, range: Range) -> ParagraphOrigin {
    ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: range,
        aggregate: range,
        kind: ParagraphOriginKind::Authored,
    }
}

fn contextual(text: &str, mode: WhiteSpace) -> ContextualSourceInput {
    let source = Source::new(text);
    let len = source.text().len();
    ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: vec![origin(&source, Range::new(0, len))],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        scopes: vec![SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            used_direction: ParagraphBaseDirection::LeftToRight,
            trim: WhiteSpaceTrim::default(),
            base_style: Style {
                white_space: mode,
                ..Style::default()
            },
        }],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events: vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ],
    }
}

#[test]
fn collapsed_space_and_tab_keep_two_zero_occurrences_inside_one_deleted_mapping() {
    use LexicalCharacterRole::{Space, Text as Scalar};
    let p = local(
        "A  \tB",
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::LineFeedDefault,
    );
    ordinary(&p, p.source(), "A B");
    let mode = WhiteSpace::Collapse;
    assert_eq!(
        characters(&p),
        [
            (0, Range::new(0, 1), mode, Scalar, Range::new(0, 1), None),
            (0, Range::new(1, 2), mode, Space, Range::new(1, 2), None),
            (
                0,
                Range::new(2, 3),
                mode,
                Space,
                Range::new(2, 2),
                Some(DeletionCause::CollapsedSpace)
            ),
            (
                0,
                Range::new(3, 4),
                mode,
                Space,
                Range::new(2, 2),
                Some(DeletionCause::CollapsedSpace)
            ),
            (0, Range::new(4, 5), mode, Scalar, Range::new(2, 3), None),
        ]
    );
    assert!(p.operations().iter().any(|piece| {
        piece.source == Range::new(2, 4)
            && piece.kind == MappingKind::Delete(DeletionCause::CollapsedSpace)
    }));
    let link = p
        .lexical
        .deletion(Range::new(2, 4), DeletionCause::CollapsedSpace);
    assert_eq!(
        link.occurrences(&p.lexical)
            .map(|record| match record.kind {
                LexicalRecordKind::Character { source, .. } => source,
                _ => unreachable!("the owner returns deleted characters only"),
            })
            .collect::<Vec<_>>(),
        [Range::new(2, 3), Range::new(3, 4)]
    );
    assert_eq!(link.original_range(&p.lexical), Range::new(2, 4));
    let tab = p
        .lexical
        .deletion(Range::new(3, 4), DeletionCause::CollapsedSpace);
    assert_eq!(tab.occurrences(&p.lexical).count(), 1);
    assert_eq!(tab.original_range(&p.lexical), Range::new(3, 4));
}

#[test]
fn segment_break_eliminations_are_distinct_from_zero_space_multiplicity() {
    use LexicalCharacterRole::{SegmentBreak, Space};
    for (text, breaks, expected, deleted) in [
        (
            "A \n B",
            DocumentSegmentBreaks::LineFeedDefault,
            "A B",
            vec![
                (
                    Range::new(1, 2),
                    Space,
                    1,
                    DeletionCause::SpaceAroundSegmentBreak,
                ),
                (
                    Range::new(3, 4),
                    Space,
                    2,
                    DeletionCause::SpaceAroundSegmentBreak,
                ),
            ],
        ),
        (
            "A\n\nB",
            DocumentSegmentBreaks::LineFeedDefault,
            "A B",
            vec![(
                Range::new(2, 3),
                SegmentBreak,
                2,
                DeletionCause::DuplicateSegmentBreak,
            )],
        ),
        (
            "甲\n乙",
            DocumentSegmentBreaks::LineFeedDefault,
            "甲乙",
            vec![(
                Range::new(1, 2),
                SegmentBreak,
                1,
                DeletionCause::ContextualSegmentBreak,
            )],
        ),
        (
            "A\r\nB",
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            "A B",
            vec![],
        ),
    ] {
        let p = local(text, WhiteSpace::Collapse, breaks);
        ordinary(&p, p.source(), expected);
        let facts = characters(&p);
        assert_eq!(
            facts
                .iter()
                .filter_map(|(_, source, _, role, at, cause)| {
                    cause.map(|cause| (*source, *role, at.start, cause))
                })
                .collect::<Vec<_>>(),
            deleted,
            "{text:?}"
        );
        assert!(
            facts
                .iter()
                .all(|fact| fact.5 != Some(DeletionCause::CollapsedSpace))
        );
        if text == "A\r\nB" {
            assert_eq!(facts.len(), 3, "the declared delimiter is one occurrence");
            assert_eq!(
                facts[1],
                (
                    0,
                    Range::new(1, 3),
                    WhiteSpace::Collapse,
                    SegmentBreak,
                    Range::new(1, 2),
                    None
                )
            );
        }
    }
}

#[test]
fn preserved_converted_discarded_and_collapsed_spaces_retain_their_real_modes() {
    use LexicalCharacterRole::{Space, Text as Scalar};
    for (mode, expected, first, tab, end) in [
        (
            WhiteSpace::Collapse,
            "A B",
            None,
            Some(DeletionCause::CollapsedSpace),
            3,
        ),
        (WhiteSpace::Preserve, "A \tB", None, None, 4),
        (WhiteSpace::PreserveSpaces, "A  B", None, None, 4),
        (
            WhiteSpace::PreserveBreaks,
            "A B",
            None,
            Some(DeletionCause::CollapsedSpace),
            3,
        ),
        (WhiteSpace::BreakSpaces, "A \tB", None, None, 4),
        (
            WhiteSpace::Discard,
            "AB",
            Some(DeletionCause::Discard),
            Some(DeletionCause::Discard),
            2,
        ),
    ] {
        let p = local("A \tB", mode, DocumentSegmentBreaks::LineFeedDefault);
        ordinary(&p, p.source(), expected);
        let first_end = if first.is_some() { 1 } else { 2 };
        let tab_end = if tab.is_some() {
            first_end
        } else {
            first_end + 1
        };
        assert_eq!(
            characters(&p),
            [
                (0, Range::new(0, 1), mode, Scalar, Range::new(0, 1), None),
                (
                    0,
                    Range::new(1, 2),
                    mode,
                    Space,
                    Range::new(1, first_end),
                    first
                ),
                (
                    0,
                    Range::new(2, 3),
                    mode,
                    Space,
                    Range::new(first_end, tab_end),
                    tab
                ),
                (
                    0,
                    Range::new(3, 4),
                    mode,
                    Scalar,
                    Range::new(end - 1, end),
                    None
                ),
            ]
        );
    }
}

#[test]
fn contextual_trim_is_genuine_elimination_even_for_preserved_spaces() {
    let mut input = contextual(" A ", WhiteSpace::Preserve);
    input.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim {
            discard_inner: true,
            ..WhiteSpaceTrim::default()
        },
    });
    input.events.insert(2, SourceProcessingEvent::EnterScope(1));
    input.events.insert(4, SourceProcessingEvent::ExitScope(1));
    let source = input.source.clone();
    let p = process_contextual_source(input).unwrap();
    ordinary(&p, &source, "A");
    assert_eq!(
        characters(&p),
        [
            (
                0,
                Range::new(0, 1),
                WhiteSpace::Preserve,
                LexicalCharacterRole::Space,
                Range::new(0, 0),
                Some(DeletionCause::Trim)
            ),
            (
                0,
                Range::new(1, 2),
                WhiteSpace::Preserve,
                LexicalCharacterRole::Text,
                Range::new(0, 1),
                None
            ),
            (
                0,
                Range::new(2, 3),
                WhiteSpace::Preserve,
                LexicalCharacterRole::Space,
                Range::new(1, 1),
                Some(DeletionCause::Trim)
            ),
        ]
    );
}

#[test]
fn continuous_origins_and_scope_markers_do_not_split_the_collapse_context() {
    let mut input = contextual("A  \tB", WhiteSpace::Collapse);
    input.origins = vec![
        origin(&input.source, Range::new(0, 2)),
        origin(&input.source, Range::new(2, 5)),
    ];
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![ParagraphBoundaryPolicy {
        definition: ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }];
    input.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim::default(),
    });
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let source = input.source.clone();
    let p = Arc::new(process_contextual_source(input).unwrap());
    ordinary(&p, &source, "A B");
    let (LexicalContext::Captured(capture), inventory) = records(&p) else {
        panic!("the real contextual operation retains its capture")
    };
    assert!(Arc::ptr_eq(
        capture,
        &p.paragraphs[0]
            .data
            .scope_topology
            .as_ref()
            .unwrap()
            .capture
    ));
    let markers = inventory
        .iter()
        .filter_map(|record| match record.kind {
            LexicalRecordKind::ScopeMarker { event } => Some(event),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(markers, [0, 4, 6, 8]);
    assert!(matches!(
        capture.events[4].event,
        SourceProcessingEvent::EnterScope(1)
    ));
    assert!(matches!(
        capture.events[6].event,
        SourceProcessingEvent::ExitScope(1)
    ));
    let facts = characters(&p);
    assert!(facts.iter().all(|fact| fact.0 == 0));
    assert_eq!(
        facts
            .iter()
            .filter(|fact| fact.5 == Some(DeletionCause::CollapsedSpace))
            .map(|fact| fact.1)
            .collect::<Vec<_>>(),
        [Range::new(2, 3), Range::new(3, 4)]
    );
    let link = p
        .lexical
        .deletion(Range::new(2, 4), DeletionCause::CollapsedSpace);
    assert_eq!(
        link.occurrences(&p.lexical).count(),
        2,
        "scope markers are not deleted SPACE occurrences"
    );
    let analyzed = AnalyzedSource::from_processed_paragraph(
        p.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.inner.projection.text, "A B");
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.source)
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(4, 5)]
    );
}

#[test]
fn actual_atomic_child_domains_keep_their_own_modes_and_canonical_event_provenance() {
    let mut input = contextual("A   I   B", WhiteSpace::Collapse);
    input.source.inline_box(InlineBox::new(
        Id::from_u64(784),
        InlineBoxKind::InFlow,
        3,
        Size::new(5.0, 10.0),
    ));
    input.origins = [Range::new(0, 3), Range::new(3, 6), Range::new(6, 9)]
        .into_iter()
        .map(|range| origin(&input.source, range))
        .collect();
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 2,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = [
        ParagraphBoundaryDefinition::Connection(0),
        ParagraphBoundaryDefinition::AtomicBefore(0),
        ParagraphBoundaryDefinition::AtomicAfter(0),
    ]
    .into_iter()
    .map(|definition| ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    })
    .collect();
    input.scopes.push(SourceProcessingScope::AtomicBlock {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: SourceProcessingTopology::Unknown,
        used_direction: ParagraphBaseDirection::LeftToRight,
        trim: WhiteSpaceTrim::default(),
        base_style: Style {
            white_space: WhiteSpace::Preserve,
            ..Style::default()
        },
        atomic: 0,
    });
    input.paragraphs.push(SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::RightToLeft,
        paragraph_strut: ParagraphStrut::DefaultFont,
    });
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::Before,
        },
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let source = input.source.clone();
    let p = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(p.source(), &source);
    assert_eq!(p.processed_text(), &Text::from("A  I  B"));
    let facts = characters(&p);
    assert_eq!(
        facts
            .iter()
            .map(|fact| (fact.0, fact.1, fact.2))
            .collect::<Vec<_>>(),
        [
            (0, Range::new(0, 1), WhiteSpace::Collapse),
            (0, Range::new(1, 2), WhiteSpace::Collapse),
            (0, Range::new(2, 3), WhiteSpace::Collapse),
            (1, Range::new(3, 4), WhiteSpace::Preserve),
            (1, Range::new(4, 5), WhiteSpace::Preserve),
            (1, Range::new(5, 6), WhiteSpace::Preserve),
            (0, Range::new(6, 7), WhiteSpace::Collapse),
            (0, Range::new(7, 8), WhiteSpace::Collapse),
            (0, Range::new(8, 9), WhiteSpace::Collapse),
        ]
    );
    assert_eq!(
        facts
            .iter()
            .filter(|fact| fact.5 == Some(DeletionCause::CollapsedSpace))
            .map(|fact| fact.1)
            .collect::<Vec<_>>(),
        [Range::new(2, 3), Range::new(7, 8)]
    );
    let (LexicalContext::Captured(capture), inventory) = records(&p) else {
        panic!("captured context")
    };
    assert!(inventory.iter().any(|record| matches!(
        record.kind,
        LexicalRecordKind::Barrier {
            origin: LexicalBarrierOrigin::CapturedEvent(3)
        }
    )));
    assert!(matches!(
        capture.events[3].event,
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::Before
        }
    ));
    assert!(p.paragraphs.iter().all(|paragraph| Arc::ptr_eq(
        capture,
        &paragraph.data.scope_topology.as_ref().unwrap().capture
    )));
    let views = p.paragraphs().collect::<Vec<_>>();
    assert_eq!(
        views[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(6, 9)]
    );
    assert_eq!(
        views[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(3, 6)]
    );
    assert_eq!(
        plaintext(views[0].plaintext_items(Affinity::After)),
        "A  B".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        plaintext(views[1].plaintext_items(Affinity::After)),
        " I ".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn identity_empty_execution_and_nonparticipating_capture_are_distinct_states() {
    let source = Source::new("A  B");
    let untouched = identity(
        source.clone(),
        Style {
            white_space: WhiteSpace::Discard,
            ..Style::default()
        },
    )
    .unwrap();
    ordinary(&untouched, &source, "A  B");
    assert!(matches!(untouched.lexical, LexicalState::Untransformed));

    let empty = local(
        "",
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::LineFeedDefault,
    );
    assert!(records(&empty).1.is_empty());
    ordinary(&empty, empty.source(), "");

    // A real Block domain requires its paragraph. Discard leaves authentic
    // nonempty source provenance without any emitted selection participant.
    let input = contextual(" \t\n", WhiteSpace::Discard);
    let original = input.source.clone();
    let p = Arc::new(process_contextual_source(input).unwrap());
    let (LexicalContext::Captured(capture), inventory) = records(&p) else {
        panic!("genuine eliminations retain their actual capture")
    };
    ordinary(&p, &original, "");
    assert_eq!(capture.scopes.len(), 1);
    assert_eq!(capture.events.len(), 5);
    assert_eq!(
        characters(&p),
        [
            (
                0,
                Range::new(0, 1),
                WhiteSpace::Discard,
                LexicalCharacterRole::Space,
                Range::new(0, 0),
                Some(DeletionCause::Discard)
            ),
            (
                0,
                Range::new(1, 2),
                WhiteSpace::Discard,
                LexicalCharacterRole::Space,
                Range::new(0, 0),
                Some(DeletionCause::Discard)
            ),
            (
                0,
                Range::new(2, 3),
                WhiteSpace::Discard,
                LexicalCharacterRole::SegmentBreak,
                Range::new(0, 0),
                Some(DeletionCause::Discard)
            ),
        ]
    );
    assert_eq!(
        inventory
            .iter()
            .filter_map(|record| match record.kind {
                LexicalRecordKind::ScopeMarker { event } => Some(event),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [0, 4]
    );
    let link = p.lexical.deletion(Range::new(0, 3), DeletionCause::Discard);
    assert_eq!(link.occurrences(&p.lexical).count(), 3);
    assert_eq!(link.original_range(&p.lexical), Range::new(0, 3));
    let paragraphs = p.paragraphs().collect::<Vec<_>>();
    assert_eq!(paragraphs.len(), 1);
    assert_eq!(paragraphs[0].source_range(), Range::new(0, 3));
    assert!(Arc::ptr_eq(
        capture,
        &p.paragraphs[0]
            .data
            .scope_topology
            .as_ref()
            .unwrap()
            .capture
    ));
    assert!(p.captured_scope(0).is_some());
    assert!(p.captured_paragraph(0).is_some());
}

#[test]
fn original_scalar_extents_and_real_bidi_roles_survive_without_glyph_classification() {
    let p = local(
        "😀\u{301}\u{2065}\u{200e}",
        WhiteSpace::Preserve,
        DocumentSegmentBreaks::LineFeedDefault,
    );
    ordinary(&p, p.source(), "😀\u{301}\u{2065}\u{200e}");
    use LexicalCharacterRole::{BidiControl, Text as Scalar};
    assert_eq!(
        characters(&p),
        [
            (
                0,
                Range::new(0, 2),
                WhiteSpace::Preserve,
                Scalar,
                Range::new(0, 2),
                None
            ),
            (
                0,
                Range::new(2, 3),
                WhiteSpace::Preserve,
                Scalar,
                Range::new(2, 3),
                None
            ),
            (
                0,
                Range::new(3, 4),
                WhiteSpace::Preserve,
                Scalar,
                Range::new(3, 4),
                None
            ),
            (
                0,
                Range::new(4, 5),
                WhiteSpace::Preserve,
                BidiControl,
                Range::new(4, 5),
                None
            ),
        ]
    );
}

#[test]
fn source_local_atomic_barrier_has_a_real_cut_without_captured_scope_authority() {
    let mut source = Source::new("A  B");
    source.inline_box(InlineBox::new(
        Id::from_u64(785),
        InlineBoxKind::InFlow,
        2,
        Size::new(0.0, 0.0),
    ));
    let p = process_source_whitespace(
        source.clone(),
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
        DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    ordinary(&p, &source, "A  B");
    let (context, inventory) = records(&p);
    assert!(matches!(context, LexicalContext::SourceLocal));
    assert!(inventory.iter().any(|record| matches!(
        record.kind,
        LexicalRecordKind::Barrier {
            origin: LexicalBarrierOrigin::SourceLocalCut(2)
        }
    )));
    assert!(inventory.iter().all(|record| !matches!(
        record.kind,
        LexicalRecordKind::ScopeMarker { .. }
            | LexicalRecordKind::Barrier {
                origin: LexicalBarrierOrigin::CapturedEvent(_)
            }
    )));
    assert!(characters(&p).iter().all(|fact| fact.5.is_none()));
    assert!(p.captured_scope(0).is_none());
}
