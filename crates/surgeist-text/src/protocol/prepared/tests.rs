use super::*;
use crate::protocol::paragraph::*;
use crate::protocol::*;
use crate::{
    Affinity, AnalyzedSource, DocumentSegmentBreaks, Font, FontCatalogueMode, Id, InlineBox,
    InlineBoxKind, Size, Source, Style, System, SystemOptions, Text, WhiteSpace,
};

fn paragraph(source: Source) -> ParagraphInput {
    let len = source.text().len();
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            ..Style::default()
        },
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: Vec::new(),
        controls: Vec::new(),
        boundary_policies: Vec::new(),
        replacements: Vec::new(),
        anchor_order: if len == 0 {
            vec![ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(0),
                affinity: Affinity::Before,
            }]
        } else {
            Vec::new()
        },
    }
}
fn system() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    system
}

fn native_atomic_preparation(
    text: &str,
    count: usize,
    local_wrap: crate::Wrap,
    ancestor_wrap: crate::Wrap,
) -> PreparedText {
    let mut source = Source::new(text);
    for occurrence in 0..count {
        source.inline_box(InlineBox::new(
            Id::from_u64(100 + u64::try_from(occurrence).unwrap()),
            InlineBoxKind::InFlow,
            1,
            Size::new(20.0, 20.0),
        ));
    }
    let mut input = paragraph(source);
    input.base_style.wrap = local_wrap;
    input.base_style.word_break = crate::WordBreak::KeepAll;
    input.anchor_order = (0..count)
        .map(|occurrence| ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(occurrence),
            affinity: Affinity::Before,
        })
        .collect();
    input.boundary_policies = (0..count)
        .flat_map(|occurrence| {
            [
                ParagraphBoundaryDefinition::AtomicBefore(occurrence),
                ParagraphBoundaryDefinition::AtomicAfter(occurrence),
            ]
        })
        .map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: local_wrap,
            following_wrap: local_wrap,
            ancestor_wrap,
        })
        .collect();
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    system()
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap()
}

fn allowed_atomic_sides(prepared: &PreparedText) -> Vec<[bool; 2]> {
    let participants: Vec<_> = prepared.participants().collect();
    prepared
        .atomic_views()
        .map(|atomic| {
            let participant = participants
                .iter()
                .position(|row| *row == PreparedParticipant::Atomic(atomic.handle()))
                .unwrap();
            [participant, participant + 1].map(|offset| {
                prepared.opportunity_views().any(|opportunity| {
                    opportunity.kind() == PreparedBreakKind::Allowed
                        && opportunity.boundary().view().participant_offset() == offset
                })
            })
        })
        .collect()
}

#[test]
fn native_atomic_inflow_issues_both_soft_breaks_despite_keep_all_text() {
    // CSS Text 3 §5.1 atomic-compat-wrap: atomic opportunities are independent
    // of the ordinary alphabetic boundary and cannot be suppressed by KeepAll.
    let prepared = native_atomic_preparation("AB", 1, crate::Wrap::Word, crate::Wrap::Word);
    assert_eq!(allowed_atomic_sides(&prepared), [[true, true]]);
}

#[test]
fn native_atomic_same_anchor_occurrences_keep_distinct_incident_breaks() {
    let prepared = native_atomic_preparation("AB", 2, crate::Wrap::Word, crate::Wrap::Word);
    let atomics: Vec<_> = prepared.atomic_views().map(|row| row.handle()).collect();
    assert_ne!(atomics[0], atomics[1]);
    assert_eq!(
        allowed_atomic_sides(&prepared),
        [[true, true], [true, true]]
    );
    // The shared middle opportunity is the boundary between actual occurrences,
    // despite every incident source endpoint being the same original index.
    let allowed: Vec<_> = prepared
        .opportunity_views()
        .filter(|row| row.kind() == PreparedBreakKind::Allowed)
        .map(|row| row.boundary().view().participant_offset())
        .collect();
    assert_eq!(allowed, [1, 2, 3]);
}

#[test]
fn native_atomic_end_of_text_still_issues_each_interior_break() {
    let prepared = native_atomic_preparation("X", 2, crate::Wrap::Word, crate::Wrap::Word);
    assert_eq!(
        allowed_atomic_sides(&prepared),
        [[true, true], [true, false]]
    );
    let opportunities: Vec<_> = prepared
        .opportunity_views()
        .map(|row| (row.boundary().view().participant_offset(), row.kind()))
        .collect();
    assert_eq!(
        opportunities,
        [
            (1, PreparedBreakKind::Allowed),
            (2, PreparedBreakKind::Allowed),
            (3, PreparedBreakKind::ParagraphEnd),
        ]
    );
}

#[test]
fn native_atomic_out_of_flow_control_creates_no_break_opportunity() {
    let mut source = Source::new("AB");
    source.inline_box(InlineBox::new(
        Id::from_u64(100),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(20.0, 20.0),
    ));
    let mut input = paragraph(source);
    input.base_style.word_break = crate::WordBreak::KeepAll;
    input.anchor_order = vec![ParagraphAnchor {
        item: ParagraphAnchorItem::Atomic(0),
        affinity: Affinity::Before,
    }];
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    let prepared = system()
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    assert_eq!(allowed_atomic_sides(&prepared), [[false, false]]);
}

#[test]
fn native_atomic_breaks_override_nbsp_glue_on_either_side() {
    // The named NBSP exception applies on either side of the atomic.
    let prepared =
        native_atomic_preparation("\u{a0}\u{a0}", 1, crate::Wrap::Word, crate::Wrap::Word);
    assert_eq!(allowed_atomic_sides(&prepared), [[true, true]]);
}

#[test]
fn native_atomic_breaks_respect_non_nbsp_gl_wj_and_zwj() {
    // U+202F is GL, U+2060 is WJ, U+200D is ZWJ in Unicode Line_Break.
    use icu_properties::{CodePointMapData, props::LineBreak};
    let classes = CodePointMapData::<LineBreak>::new();
    let observed: Vec<_> = [
        ("\u{202f}\u{202f}", 0x202f, LineBreak::Glue),
        ("\u{2060}\u{2060}", 0x2060, LineBreak::WordJoiner),
        ("\u{200d}\u{200d}", 0x200d, LineBreak::ZWJ),
    ]
    .into_iter()
    .map(|(text, code_point, class)| {
        assert_eq!(classes.get32(code_point), class);
        let prepared = native_atomic_preparation(text, 1, crate::Wrap::Word, crate::Wrap::Word);
        allowed_atomic_sides(&prepared)
    })
    .collect();
    assert_eq!(observed, vec![vec![[false, false]]; 3]);
}

#[test]
fn native_atomic_break_permission_comes_from_captured_common_ancestor() {
    // The atomic boundary's owner remains its captured NCA even when the
    // preceding character is SPACE with its own local wrapping permission.
    let wrapping = native_atomic_preparation(" A", 1, crate::Wrap::None, crate::Wrap::Word);
    let no_wrapping = native_atomic_preparation(" A", 1, crate::Wrap::Word, crate::Wrap::None);
    assert_eq!(
        [
            allowed_atomic_sides(&wrapping),
            allowed_atomic_sides(&no_wrapping),
        ],
        [vec![[true, true]], vec![[false, false]]]
    );
}

#[test]
fn native_atomic_glue_suppression_is_specific_to_the_incident_side() {
    for text in ["A\u{202f}", "A\u{2060}", "A\u{200d}"] {
        let prepared = native_atomic_preparation(text, 1, crate::Wrap::Word, crate::Wrap::Word);
        assert_eq!(allowed_atomic_sides(&prepared), [[true, false]], "{text:?}");
    }
}

#[test]
fn native_atomic_pair_is_not_adjacent_to_the_following_glue_character() {
    let prepared = native_atomic_preparation("A\u{202f}", 2, crate::Wrap::Word, crate::Wrap::Word);
    assert_eq!(
        allowed_atomic_sides(&prepared),
        [[true, true], [true, false]]
    );
}

#[test]
fn native_atomic_transparent_occurrences_do_not_reintroduce_a_forbidden_break() {
    let observed: Vec<_> = [true, false]
        .into_iter()
        .map(|out_of_flow| {
            let mut source = Source::new("A\u{202f}");
            source.inline_box(InlineBox::new(
                Id::from_u64(100),
                InlineBoxKind::InFlow,
                1,
                Size::new(20.0, 20.0),
            ));
            if out_of_flow {
                source.inline_box(InlineBox::new(
                    Id::from_u64(101),
                    InlineBoxKind::OutOfFlow,
                    1,
                    Size::new(20.0, 20.0),
                ));
            }
            let mut input = paragraph(source);
            input.anchor_order = vec![
                ParagraphAnchor {
                    item: ParagraphAnchorItem::Atomic(0),
                    affinity: Affinity::Before,
                },
                ParagraphAnchor {
                    item: if out_of_flow {
                        ParagraphAnchorItem::Atomic(1)
                    } else {
                        ParagraphAnchorItem::Control(0)
                    },
                    affinity: Affinity::Before,
                },
            ];
            if !out_of_flow {
                input.controls.push(ParagraphControl {
                    at: 1,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                });
            }
            input.boundary_policies = [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect();
            let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
            let prepared = system()
                .prepare(
                    &analyzed,
                    PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
                )
                .unwrap();
            prepared
                .opportunity_views()
                .map(|row| (row.boundary().view().participant_offset(), row.kind()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        observed,
        vec![
            vec![
                (1, PreparedBreakKind::Allowed),
                (4, PreparedBreakKind::ParagraphEnd),
            ];
            2
        ]
    );
}

#[derive(Clone)]
enum TransparentAtomicNeighbor {
    OutOfFlow,
    InlineBoundary,
    EmptyOrigin,
    Directional(ParagraphControlKind),
}

fn atomic_transparent_input(
    text: &str,
    at: usize,
    transparent: TransparentAtomicNeighbor,
    transparent_first: bool,
) -> ParagraphInput {
    let mut source = Source::new(text);
    source.inline_box(InlineBox::new(
        Id::from_u64(100),
        InlineBoxKind::InFlow,
        at,
        Size::new(20.0, 20.0),
    ));
    if matches!(transparent, TransparentAtomicNeighbor::OutOfFlow) {
        source.inline_box(InlineBox::new(
            Id::from_u64(101),
            InlineBoxKind::OutOfFlow,
            at,
            Size::new(20.0, 20.0),
        ));
    }
    let mut input = paragraph(source);
    let transparent_item = match transparent {
        TransparentAtomicNeighbor::OutOfFlow => ParagraphAnchorItem::Atomic(1),
        TransparentAtomicNeighbor::InlineBoundary | TransparentAtomicNeighbor::Directional(_) => {
            input.controls.push(ParagraphControl {
                at,
                kind: match transparent {
                    TransparentAtomicNeighbor::Directional(kind) => kind,
                    _ => ParagraphControlKind::InlineBoundaryStart,
                },
            });
            ParagraphAnchorItem::Control(0)
        }
        TransparentAtomicNeighbor::EmptyOrigin => {
            let len = input.source.text().len();
            input.origins.clear();
            if at != 0 {
                input.origins.push(ParagraphOrigin {
                    text_node: None,
                    source: input.source.clone(),
                    local: Range::new(0, at),
                    aggregate: Range::new(0, at),
                    kind: ParagraphOriginKind::Authored,
                });
            }
            let empty = input.origins.len();
            input.origins.push(ParagraphOrigin {
                text_node: None,
                source: Source::new(""),
                local: Range::new(0, 0),
                aggregate: Range::new(at, at),
                kind: ParagraphOriginKind::Authored,
            });
            if at != len {
                input.origins.push(ParagraphOrigin {
                    text_node: None,
                    source: input.source.clone(),
                    local: Range::new(at, len),
                    aggregate: Range::new(at, len),
                    kind: ParagraphOriginKind::Authored,
                });
            }
            input.connections = vec![
                ParagraphConnection {
                    at,
                    kind: ParagraphConnectionKind::Continuous,
                };
                input.origins.len() - 1
            ];
            ParagraphAnchorItem::EmptyOrigin(empty)
        }
    };
    let mut anchors = [ParagraphAnchorItem::Atomic(0), transparent_item];
    if transparent_first {
        anchors.reverse();
    }
    input.anchor_order = anchors
        .into_iter()
        .map(|item| ParagraphAnchor {
            item,
            affinity: Affinity::Before,
        })
        .collect();
    input.boundary_policies = [
        ParagraphBoundaryDefinition::AtomicBefore(0),
        ParagraphBoundaryDefinition::AtomicAfter(0),
    ]
    .into_iter()
    .chain((0..input.connections.len()).map(ParagraphBoundaryDefinition::Connection))
    .map(|definition| ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: crate::Wrap::Word,
        following_wrap: crate::Wrap::Word,
        ancestor_wrap: crate::Wrap::Word,
    })
    .collect();
    input
}

fn prepare_opportunity_inventory(input: ParagraphInput) -> Vec<(usize, PreparedBreakKind)> {
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    let prepared = system()
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    prepared
        .opportunity_views()
        .map(|row| (row.boundary().view().participant_offset(), row.kind()))
        .collect()
}

#[test]
fn native_hard_separator_precedence_uses_the_existing_transparent_neighbors() {
    for transparent in [
        TransparentAtomicNeighbor::OutOfFlow,
        TransparentAtomicNeighbor::InlineBoundary,
        TransparentAtomicNeighbor::EmptyOrigin,
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::InlineBoundaryEnd),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::LeftToRightEmbedding),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::RightToLeftEmbedding),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::LeftToRightOverride),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::RightToLeftOverride),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::PopDirectionalFormat),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::LeftToRightIsolate),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::RightToLeftIsolate),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::FirstStrongIsolate),
        TransparentAtomicNeighbor::Directional(ParagraphControlKind::PopDirectionalIsolate),
    ] {
        let mut input = atomic_transparent_input("X", 0, transparent, false);
        let control = input.controls.len();
        input.controls.push(ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::ForcedBreak,
        });
        input.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Control(control),
            affinity: Affinity::Before,
        });
        assert_eq!(
            prepare_opportunity_inventory(input),
            [
                (3, PreparedBreakKind::Forced),
                (4, PreparedBreakKind::ParagraphEnd),
            ]
        );
    }
}

#[test]
fn native_hard_separator_precedence_keeps_earlier_soft_control_owners() {
    for (kind, expected) in [
        (
            ParagraphControlKind::OptionalBreak {
                wrap: crate::Wrap::Word,
            },
            PreparedBreakKind::Allowed,
        ),
        (
            ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(paragraph(Source::new("")).base_style),
            },
            PreparedBreakKind::Discretionary,
        ),
    ] {
        let mut input = paragraph(Source::new("XY"));
        input.controls = vec![
            ParagraphControl { at: 0, kind },
            ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::ForcedBreak,
            },
        ];
        input.anchor_order = (0..2)
            .map(|control| ParagraphAnchor {
                item: ParagraphAnchorItem::Control(control),
                affinity: Affinity::Before,
            })
            .collect();
        if expected == PreparedBreakKind::Discretionary {
            input.replacements.push(
                ParagraphReplacement::resolved_native_logical_before(
                    ParagraphReplacementDefinition::Control {
                        control: 0,
                        origin: None,
                    },
                    0,
                    vec![Text::from("-")],
                )
                .unwrap(),
            );
        }
        let mut expected_inventory = vec![
            (3, PreparedBreakKind::Forced),
            (4, PreparedBreakKind::ParagraphEnd),
        ];
        if expected == PreparedBreakKind::Allowed {
            expected_inventory.insert(0, (1, expected));
        }
        assert_eq!(prepare_opportunity_inventory(input), expected_inventory);
    }
}

#[test]
fn native_hard_separator_precedence_preserves_mandatory_cuts_before_transparent_rows() {
    let mut input = paragraph(Source::new("X"));
    input.controls = [
        ParagraphControlKind::ForcedBreak,
        ParagraphControlKind::InlineBoundaryStart,
        ParagraphControlKind::ForcedBreak,
    ]
    .map(|kind| ParagraphControl { at: 0, kind })
    .to_vec();
    input.anchor_order = (0..3)
        .map(|control| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(control),
            affinity: Affinity::Before,
        })
        .collect();
    assert_eq!(
        prepare_opportunity_inventory(input),
        [
            (1, PreparedBreakKind::Forced),
            (3, PreparedBreakKind::Forced),
            (4, PreparedBreakKind::ParagraphEnd),
        ]
    );
}

#[test]
fn native_atomic_transparent_internal_cuts_lend_the_same_permitted_relation() {
    for transparent in [
        TransparentAtomicNeighbor::OutOfFlow,
        TransparentAtomicNeighbor::InlineBoundary,
        TransparentAtomicNeighbor::EmptyOrigin,
    ] {
        let input = atomic_transparent_input("AB", 1, transparent, false);
        assert_eq!(
            prepare_opportunity_inventory(input),
            [
                (1, PreparedBreakKind::Allowed),
                (2, PreparedBreakKind::Allowed),
                (3, PreparedBreakKind::Allowed),
                (4, PreparedBreakKind::ParagraphEnd),
            ]
        );
    }
}

#[test]
fn native_atomic_directional_controls_do_not_hide_incident_glue() {
    for kind in [
        ParagraphControlKind::LeftToRightEmbedding,
        ParagraphControlKind::RightToLeftEmbedding,
        ParagraphControlKind::LeftToRightOverride,
        ParagraphControlKind::RightToLeftOverride,
        ParagraphControlKind::PopDirectionalFormat,
        ParagraphControlKind::LeftToRightIsolate,
        ParagraphControlKind::RightToLeftIsolate,
        ParagraphControlKind::FirstStrongIsolate,
        ParagraphControlKind::PopDirectionalIsolate,
    ] {
        for text in ["A\u{202f}", "A\u{2060}", "A\u{200d}"] {
            assert_eq!(
                prepare_opportunity_inventory(atomic_transparent_input(
                    text,
                    1,
                    TransparentAtomicNeighbor::Directional(kind.clone()),
                    false,
                )),
                [
                    (1, PreparedBreakKind::Allowed),
                    (4, PreparedBreakKind::ParagraphEnd),
                ],
                "{kind:?} {text:?}"
            );
        }
    }
}

#[test]
fn native_atomic_transparent_semantic_edges_create_no_extra_soft_line() {
    for transparent in [
        TransparentAtomicNeighbor::OutOfFlow,
        TransparentAtomicNeighbor::InlineBoundary,
        TransparentAtomicNeighbor::EmptyOrigin,
    ] {
        assert_eq!(
            prepare_opportunity_inventory(atomic_transparent_input(
                "X",
                1,
                transparent.clone(),
                false
            )),
            [
                (1, PreparedBreakKind::Allowed),
                (3, PreparedBreakKind::ParagraphEnd),
            ]
        );
        assert_eq!(
            prepare_opportunity_inventory(atomic_transparent_input("X", 0, transparent, true)),
            [
                (2, PreparedBreakKind::Allowed),
                (3, PreparedBreakKind::ParagraphEnd),
            ]
        );
    }
}

#[test]
fn native_atomic_transparent_relation_rejects_disagreeing_captured_ancestors() {
    let mut input =
        atomic_transparent_input("AB", 1, TransparentAtomicNeighbor::InlineBoundary, false);
    input.source.inline_box(InlineBox::new(
        Id::from_u64(101),
        InlineBoxKind::InFlow,
        1,
        Size::new(20.0, 20.0),
    ));
    input.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Atomic(1),
        affinity: Affinity::Before,
    });
    input.boundary_policies.extend([
        ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::AtomicBefore(1),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::None,
        },
        ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::AtomicAfter(1),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        },
    ]);
    let error = AnalyzedSource::from_paragraph(input).unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&crate::ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::InvalidBoundaryPolicy { policy: 2, at: 1 },
        })
    );
}

#[test]
fn native_atomic_collapse_space_permission_is_local_only_before_the_atomic() {
    let observed: Vec<_> = [WhiteSpace::Preserve, WhiteSpace::Collapse]
        .into_iter()
        .flat_map(|white_space| {
            [
                (crate::Wrap::Word, crate::Wrap::None),
                (crate::Wrap::None, crate::Wrap::Word),
            ]
            .into_iter()
            .map(move |(local_wrap, ancestor_wrap)| {
                let mut source = Source::new("A B");
                source.inline_box(InlineBox::new(
                    Id::from_u64(100),
                    InlineBoxKind::InFlow,
                    2,
                    Size::new(20.0, 20.0),
                ));
                let mut input = paragraph(source);
                input.base_style.white_space = white_space;
                input.base_style.wrap = local_wrap;
                input.anchor_order = vec![ParagraphAnchor {
                    item: ParagraphAnchorItem::Atomic(0),
                    affinity: Affinity::Before,
                }];
                input.boundary_policies = [
                    ParagraphBoundaryDefinition::AtomicBefore(0),
                    ParagraphBoundaryDefinition::AtomicAfter(0),
                ]
                .into_iter()
                .map(|definition| ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: local_wrap,
                    following_wrap: local_wrap,
                    ancestor_wrap,
                })
                .collect();
                let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
                let prepared = system()
                    .prepare(
                        &analyzed,
                        PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance)
                            .unwrap(),
                    )
                    .unwrap();
                allowed_atomic_sides(&prepared)
            })
        })
        .collect();
    assert_eq!(
        observed,
        [
            vec![[false, false]],
            vec![[true, true]],
            vec![[true, false]],
            vec![[true, true]],
        ]
    );
}

#[test]
fn native_connection_space_owner_depends_on_actual_disappearance() {
    let observed: Vec<_> = [WhiteSpace::Preserve, WhiteSpace::Collapse]
        .into_iter()
        .flat_map(|white_space| {
            [
                (crate::Wrap::Word, crate::Wrap::None),
                (crate::Wrap::None, crate::Wrap::Word),
            ]
            .into_iter()
            .map(move |(local_wrap, ancestor_wrap)| {
                let mut input = paragraph(Source::new("A B"));
                input.base_style.white_space = white_space;
                input.base_style.wrap = local_wrap;
                input.origins = vec![
                    ParagraphOrigin {
                        text_node: None,
                        source: Source::new("A "),
                        local: Range::new(0, 2),
                        aggregate: Range::new(0, 2),
                        kind: ParagraphOriginKind::Authored,
                    },
                    ParagraphOrigin {
                        text_node: None,
                        source: Source::new("B"),
                        local: Range::new(0, 1),
                        aggregate: Range::new(2, 3),
                        kind: ParagraphOriginKind::Authored,
                    },
                ];
                input.connections = vec![ParagraphConnection {
                    at: 2,
                    kind: ParagraphConnectionKind::Continuous,
                }];
                input.boundary_policies = vec![ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: local_wrap,
                    following_wrap: local_wrap,
                    ancestor_wrap,
                }];
                let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
                let prepared = system()
                    .prepare(
                        &analyzed,
                        PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance)
                            .unwrap(),
                    )
                    .unwrap();
                prepared.opportunity_views().any(|row| {
                    row.kind() == PreparedBreakKind::Allowed
                        && row.boundary().view().source_before().get() == 2
                })
            })
        })
        .collect();
    assert_eq!(observed, [false, true, true, false]);
}

fn context(end: LineEnd) -> LineContext {
    LineContext {
        base_direction: ParagraphBaseDirection::LeftToRight,
        orientation: LineOrientation::Horizontal,
        start: LineStart::Paragraph,
        end,
        available_inline: None,
        indent: 0.0,
        alignment: crate::Alignment::Start,
        tabs: TabContext::NoTabs,

        spacing: SpacingPolicy::Spaces,
    }
}
fn select_whole(prepared: &PreparedText) -> SelectedLine {
    let start = prepared.boundary_views().next().unwrap().handle();
    let termination = prepared
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    prepared
        .select(LineSelectionRequest {
            incoming: None,
            start,
            end: termination.view().boundary(),
            participants: prepared.participants().collect(),
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative: None,
            context: context(LineEnd::Paragraph),
            atomics: Vec::new(),
            line_strut: None,
            controls: prepared
                .control_views()
                .map(|control| ControlAllocation {
                    occurrence: control.handle(),
                    strut: None,
                })
                .collect(),
        })
        .unwrap()
}

#[test]
fn discarded_original_coverage_and_distinct_empty_occurrences_survive_selection() {
    let mut input = paragraph(Source::new(" A   B"));
    input.base_style.white_space = WhiteSpace::Discard;
    input.origins = vec![
        ParagraphOrigin {
            text_node: None,
            source: Source::new(" A "),
            local: Range::new(0, 3),
            aggregate: Range::new(0, 3),
            kind: ParagraphOriginKind::Authored,
        },
        ParagraphOrigin {
            text_node: None,
            source: Source::new(""),
            local: Range::new(0, 0),
            aggregate: Range::new(3, 3),
            kind: ParagraphOriginKind::Generated,
        },
        ParagraphOrigin {
            text_node: None,
            source: Source::new(""),
            local: Range::new(0, 0),
            aggregate: Range::new(3, 3),
            kind: ParagraphOriginKind::Authored,
        },
        ParagraphOrigin {
            text_node: None,
            source: Source::new("  B"),
            local: Range::new(0, 3),
            aggregate: Range::new(3, 6),
            kind: ParagraphOriginKind::Authored,
        },
    ];
    input.connections = vec![
        ParagraphConnection {
            at: 3,
            kind: ParagraphConnectionKind::Continuous
        };
        3
    ];
    input.boundary_policies = (0..3)
        .map(|i| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    input.anchor_order = vec![
        ParagraphAnchor {
            item: ParagraphAnchorItem::EmptyOrigin(1),
            affinity: Affinity::Before,
        },
        ParagraphAnchor {
            item: ParagraphAnchorItem::EmptyOrigin(2),
            affinity: Affinity::Before,
        },
    ];
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    assert_eq!(
        analyzed.processed_source().processed_text().code_units(),
        [0x41, 0x42]
    );
    let mut system = system();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    let selected = select_whole(&prepared);
    assert_eq!(selected.inner.presentation.source, Range::new(0, 6));
    assert_eq!(selected.inner.presentation.projection.text, "AB");
    let associations = selected_origin_intersections(&selected.inner);
    let empty: Vec<_> = associations
        .iter()
        .filter_map(|row| {
            row.source
                .filter(|v| v.aggregate.is_empty())
                .map(|v| v.origin)
        })
        .collect();
    assert_eq!(empty, [1, 2]);
    let covered: usize = associations
        .iter()
        .filter_map(|row| row.source)
        .map(|row| row.aggregate.len())
        .sum();
    assert_eq!(covered, 6);
    drop(system);
    drop(analyzed);
    let (shaped, mapping) = crate::engine::shape::finalize(
        &selected.inner,
        &mut crate::engine::shape::ShapeContext::default(),
    )
    .unwrap();
    assert_eq!(shaped.glyphs.len(), 2);
    assert!(std::sync::Arc::ptr_eq(
        &mapping,
        &selected.inner.presentation.mapping
    ));
    assert_eq!(
        shaped
            .items
            .iter()
            .filter(|v| matches!(v, ShapedItem::Anchor { .. }))
            .count(),
        5
    );
}

#[test]
fn explicit_transparent_half_pair_affinity_does_not_split_but_inflow_really_does() {
    for (kind, affinity, expected_atoms, expected_anchor) in [
        (InlineBoxKind::OutOfFlow, Affinity::Before, 1, 0),
        (InlineBoxKind::OutOfFlow, Affinity::After, 1, 1),
        (InlineBoxKind::InFlow, Affinity::Before, 2, 1),
        (InlineBoxKind::InFlow, Affinity::After, 2, 1),
    ] {
        let mut source = Source::new(Text::from_code_units(vec![0xd840, 0xdc00]));
        source.inline_box(InlineBox::new(
            Id::from_u64(5),
            kind,
            1,
            Size::new(2.0, 3.0),
        ));
        let mut input = paragraph(source);
        input.anchor_order = vec![ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(0),
            affinity,
        }];
        if kind == InlineBoxKind::InFlow {
            input.boundary_policies = [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect();
        }
        let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
        assert_eq!(analyzed.inner.projection.atoms.len(), expected_atoms);
        assert_eq!(
            anchor_atom(
                0,
                &analyzed.inner.paragraph,
                analyzed.source().boxes(),
                &analyzed.inner.projection
            ),
            expected_anchor
        );
        if expected_atoms == 1 {
            assert_eq!(analyzed.inner.projection.atoms[0].code_point, 0x20000);
        } else {
            assert_eq!(
                analyzed
                    .inner
                    .projection
                    .atoms
                    .iter()
                    .map(|a| a.code_point)
                    .collect::<Vec<_>>(),
                [0xd840, 0xdc00]
            );
        }
    }
}

#[test]
fn selected_source_replacement_is_actual_retained_text_with_zero_authored_extent() {
    let mut input = paragraph(Source::new("AX\u{ad}BBB"));
    input.replacements = vec![
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(2, 3),
            },
            3,
            vec![Text::from("- ")],
        )
        .unwrap(),
    ];
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    let mut system = system();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    let alternative = prepared.alternative_views().next().unwrap().handle();
    let termination = alternative.view().opportunity();
    let end = termination.view().boundary();
    let count = end.view().participant_offset();
    let selected = prepared
        .select(LineSelectionRequest {
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end,
            participants: prepared.participants().take(count).collect(),
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative: Some(alternative),
            context: context(LineEnd::Soft),
            atomics: Vec::new(),
            line_strut: None,
            controls: prepared
                .control_views()
                .map(|control| ControlAllocation {
                    occurrence: control.handle(),
                    strut: None,
                })
                .collect(),
        })
        .unwrap();
    assert_eq!(selected.inner.presentation.projection.text, "AX- ");
    assert_eq!(selected.inner.presentation.source, Range::new(0, 3));
    let replacement: Vec<_> = selected
        .inner
        .presentation
        .units
        .iter()
        .filter(|u| matches!(u.kind, PresentationUnitKind::Replacement { .. }))
        .collect();
    assert_eq!(replacement.len(), 2);
    assert!(
        replacement
            .iter()
            .all(|u| u.source == Range::new(3, 3) && u.participant.is_none())
    );
    assert!(replacement[1].eligible_space);
    assert_eq!(replacement[1].presentation, Range::new(3, 4));
    system.refresh_fonts().unwrap();
    drop(system);
    let (shaped, _) = crate::engine::shape::finalize(
        &selected.inner,
        &mut crate::engine::shape::ShapeContext::default(),
    )
    .unwrap();
    assert!(
        shaped
            .runs
            .iter()
            .all(|run| run.font.catalogue_identity() == prepared.font_catalogue_identity())
    );
    assert_eq!(
        shaped
            .glyphs
            .iter()
            .filter(|g| g.source == Range::new(3, 3))
            .count(),
        2
    );
}

#[test]
fn source_less_discretionary_control_and_empty_replacement_keep_real_owners() {
    let mut input = paragraph(Source::new("AXXXX"));
    input.anchor_order.clear();
    input.controls = vec![ParagraphControl {
        at: 2,
        kind: ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(paragraph(Source::new("")).base_style),
        },
    }];
    input.anchor_order = vec![ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: Affinity::Before,
    }];
    input.replacements = vec![
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Control {
                control: 0,
                origin: None,
            },
            2,
            vec![Text::from("")],
        )
        .unwrap(),
    ];
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    let mut system = system();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    let alternative = prepared.alternative_views().next().unwrap().handle();
    let termination = alternative.view().opportunity();
    let selected = prepared
        .select(LineSelectionRequest {
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            participants: prepared
                .participants()
                .take(termination.view().boundary().view().participant_offset())
                .collect(),
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative: Some(alternative),
            context: context(LineEnd::Soft),
            atomics: Vec::new(),
            line_strut: None,
            controls: prepared
                .control_views()
                .map(|control| ControlAllocation {
                    occurrence: control.handle(),
                    strut: None,
                })
                .collect(),
        })
        .unwrap();
    let fragments = selected_origin_intersections(&selected.inner);
    assert_eq!(fragments.len(), 4);
    let fragments: Vec<_> = fragments
        .into_iter()
        .filter(|f| !matches!(f.owner, FragmentOwner::Origin(_)))
        .collect();
    assert!(matches!(fragments[0].owner, FragmentOwner::Alternative(0)));
    assert!(matches!(fragments[1].owner, FragmentOwner::Control(0)));
    assert!(fragments.iter().all(|f| f.source.is_none()));
    let (shaped, _) = crate::engine::shape::finalize(
        &selected.inner,
        &mut crate::engine::shape::ShapeContext::default(),
    )
    .unwrap();
    assert_eq!(shaped.glyphs.len(), 2);
    assert_eq!(
        shaped
            .items
            .iter()
            .filter(|v| matches!(v, ShapedItem::Anchor { .. }))
            .count(),
        2
    );
}

#[test]
fn empty_native_forced_control_keeps_empty_host_and_semantic_atoms() {
    let mut input = paragraph(Source::new(""));
    input.origins.clear();
    input.controls = vec![ParagraphControl {
        at: 0,
        kind: ParagraphControlKind::ForcedBreak,
    }];
    input.anchor_order = vec![ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: Affinity::Before,
    }];
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    assert!(analyzed.inner.host_projection.atoms.is_empty());
    assert!(analyzed.inner.projection.atoms.is_empty());
    assert!(analyzed.processed_source().processed_text().is_empty());
    assert_eq!(analyzed.inner.analysis.control_levels.len(), 1);
}

#[test]
fn raw_standalone_crlf_forced_items_do_not_split_native_normalized_delimiter() {
    let mut system = system();
    let source = Source::new("\r\n");
    let raw = AnalyzedSource::new(source.clone(), paragraph(source.clone()).base_style).unwrap();
    let prepared = system
        .prepare(
            &raw,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    assert_eq!(
        prepared
            .inner
            .tables
            .boundaries
            .iter()
            .map(|b| b.source_before)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let forced: Vec<_> = prepared
        .inner
        .tables
        .opportunities
        .iter()
        .filter(|o| o.kind == PreparedBreakKind::Forced)
        .map(|o| prepared.inner.tables.boundaries[o.boundary].source_before)
        .collect();
    assert_eq!(forced, [1, 2]);
    assert_eq!(raw.graphemes().collect::<Vec<_>>(), [Range::new(0, 2)]);

    let mut input = paragraph(source);
    input.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(0, 2)]);
    let native = AnalyzedSource::from_paragraph(input).unwrap();
    assert_eq!(
        native.processed_source().processed_text().code_units(),
        [0xa]
    );
    let prepared = system
        .prepare(
            &native,
            PreparationOptions::try_new(1.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    assert_eq!(
        prepared
            .inner
            .tables
            .boundaries
            .iter()
            .map(|b| b.source_before)
            .collect::<Vec<_>>(),
        [0, 2]
    );
}
