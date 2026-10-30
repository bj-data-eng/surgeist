//! Existing-callable material source continuity witness. Raw paragraph storage
//! markers provide neither continuity nor a disconnected relationship.

use crate::*;
use std::sync::Arc;

fn input(
    text: &str,
    mode: WhiteSpace,
    relationship: Option<ParagraphConnectionKind>,
) -> ContextualSourceInput {
    let mut input = super::input(text, mode);
    input.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
    ];
    input.joint_flows.clear();
    input.connections.clear();
    input.boundary_policies.clear();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
    ];
    if let Some(kind) = relationship {
        input.connections.push(SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind,
        });
        input.events.push(SourceProcessingEvent::Connection(0));
        if kind == ParagraphConnectionKind::Continuous {
            input
                .boundary_policies
                .push(super::policy(ParagraphBoundaryDefinition::Connection(0)));
        }
    }
    input.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(0),
    ]);
    input
}

#[test]
fn unknown_source_relationship_refuses_material_cross_paragraph_space_collapse() {
    // Each raw paragraph owns one actual SPACE. Neither a source relationship
    // nor joint-analysis authority was supplied. Global Phase I must not guess
    // whether the second occurrence collapses against the first.
    let error = process_contextual_source(input("  ", WhiteSpace::Collapse, None)).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    // Unavailable contextual source processing retains its typed source category.
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceProcessing { .. })
    ));
}

#[test]
fn explicit_relationships_and_nonmaterial_unknown_context_keep_real_source_outputs() {
    for (text, mode, relationship, expected) in [
        (
            "  ",
            WhiteSpace::Collapse,
            Some(ParagraphConnectionKind::Continuous),
            " ",
        ),
        (
            "  ",
            WhiteSpace::Collapse,
            Some(ParagraphConnectionKind::Disconnected),
            "  ",
        ),
        ("AB", WhiteSpace::Collapse, None, "AB"),
        ("  ", WhiteSpace::Preserve, None, "  "),
        ("  ", WhiteSpace::Discard, None, ""),
    ] {
        let input = input(text, mode, relationship);
        let source = input.source.clone();
        let processed = Arc::new(process_contextual_source(input).unwrap());
        assert_eq!(processed.source(), &source);
        assert_eq!(processed.processed_text(), &Text::from(expected));
        assert!(processed.captured_joint_flow(0).is_none());
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(raw.len(), 2);
        for (ordinal, range) in [Range::new(0, 1), Range::new(1, 2)].into_iter().enumerate() {
            assert_eq!(raw[ordinal].source_range(), range);
            assert_eq!(raw[ordinal].source_pieces().collect::<Vec<_>>(), [range]);
            assert!(Arc::ptr_eq(raw[ordinal].processed_source(), &processed));
            assert!(raw[ordinal].data().connections.is_empty());
            assert!(raw[ordinal].data().boundary_policies.is_empty());
            assert_eq!(
                processed.captured_paragraph(ordinal).unwrap().joint_flow(),
                None
            );
        }
    }
}

fn split(
    text: impl Into<Text>,
    at: usize,
    mode: WhiteSpace,
    relationship: Option<ParagraphConnectionKind>,
) -> ContextualSourceInput {
    let mut request = input("AB", mode, relationship);
    request.source = Source::new(text);
    request.origins = [
        Range::new(0, at),
        Range::new(at, request.source.text().len()),
    ]
    .into_iter()
    .map(|range| super::origin(&request.source, range))
    .collect();
    request
}

fn missing(
    request: ContextualSourceInput,
    before_origin: usize,
    after_origin: usize,
    operation: SourceContinuityOperation,
) {
    let error = process_contextual_source(request).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin,
                after_origin,
                operation,
            },
        })
    );
}

fn deleted(processed: &ProcessedSource) -> Vec<(Range, crate::processing::DeletionCause)> {
    use crate::processing::lexical_inventory::{
        LexicalDisposition, LexicalRecordKind, LexicalState,
    };
    let LexicalState::Executed { records, .. } = &processed.lexical else {
        panic!("actual executed source")
    };
    records
        .iter()
        .filter_map(|record| match record.kind {
            LexicalRecordKind::Character {
                source,
                disposition: LexicalDisposition::Deleted { cause, .. },
                ..
            } => Some((source, cause)),
            _ => None,
        })
        .collect()
}

fn character_ranges(processed: &ProcessedSource) -> Vec<Range> {
    use crate::processing::lexical_inventory::{LexicalRecordKind, LexicalState};
    let LexicalState::Executed { records, .. } = &processed.lexical else {
        panic!("actual executed source")
    };
    records
        .iter()
        .filter_map(|record| match record.kind {
            LexicalRecordKind::Character { source, .. } => Some(source),
            _ => None,
        })
        .collect()
}

#[test]
fn ordinary_nonspace_neighbors_reset_collapse_without_requiring_unknown_context() {
    for value in [" A", "A "] {
        let request = split(value, 1, WhiteSpace::Collapse, None);
        let source = request.source.clone();
        let processed = process_contextual_source(request).unwrap();
        assert_eq!(processed.source(), &source);
        assert_eq!(processed.processed_text(), &Text::from(value));
        assert!(deleted(&processed).is_empty());
    }
}

#[test]
fn actual_tab_around_break_duplicate_break_and_bidi_transparent_folds_need_continuity() {
    use crate::processing::DeletionCause as D;
    // Pinned CSS Text Phase I: SPACE/TAB around a break is removed, TAB
    // becomes SPACE, following duplicate SPACE/break disappears, and actual
    // Bidi_Control characters are transparent. See the retained CSS Text 4
    // whitespace capture and crate docs/reference.md#lexical-collapse.
    for (value, zero, cause, continuous, disconnected) in [
        (" \t", Range::new(1, 2), D::CollapsedSpace, " ", "  "),
        (
            " \n",
            Range::new(0, 1),
            D::SpaceAroundSegmentBreak,
            " ",
            "  ",
        ),
        (
            "\n ",
            Range::new(1, 2),
            D::SpaceAroundSegmentBreak,
            " ",
            "  ",
        ),
        (
            "\n\n",
            Range::new(1, 2),
            D::DuplicateSegmentBreak,
            " ",
            "  ",
        ),
        (
            " \u{200e} ",
            Range::new(2, 3),
            D::CollapsedSpace,
            " \u{200e}",
            " \u{200e} ",
        ),
    ] {
        missing(
            split(value, 1, WhiteSpace::Collapse, None),
            0,
            1,
            SourceContinuityOperation::WhiteSpace,
        );
        for (kind, expected, expected_deletion) in [
            (
                ParagraphConnectionKind::Continuous,
                continuous,
                Some((zero, cause)),
            ),
            (ParagraphConnectionKind::Disconnected, disconnected, None),
        ] {
            let request = split(value, 1, WhiteSpace::Collapse, Some(kind));
            let source = request.source.clone();
            let processed = process_contextual_source(request).unwrap();
            assert_eq!(processed.source(), &source);
            assert_eq!(
                processed.processed_text(),
                &Text::from(expected),
                "{value:?}/{kind:?}"
            );
            assert_eq!(
                deleted(&processed),
                expected_deletion.into_iter().collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn conditional_segment_breaks_require_only_context_that_can_change_the_native_decision() {
    // CSS leaves this contextual choice to the UA. The adopted native policy
    // (crate docs/reference.md#lexical-collapse) removes a break beside ZWSP
    // or between two F/W/H non-Hangul scalars. These are not browser-parity tests.
    for (value, continuous, disconnected) in [
        ("中\n中", "中中", "中 中"),
        ("A\n\u{200b}", "A\u{200b}", "A \u{200b}"),
    ] {
        missing(
            split(value, 2, WhiteSpace::Collapse, None),
            0,
            1,
            SourceContinuityOperation::WhiteSpace,
        );
        for (kind, expected) in [
            (ParagraphConnectionKind::Continuous, continuous),
            (ParagraphConnectionKind::Disconnected, disconnected),
        ] {
            let processed =
                process_contextual_source(split(value, 2, WhiteSpace::Collapse, Some(kind)))
                    .unwrap();
            assert_eq!(processed.processed_text(), &Text::from(expected));
        }
    }
    // The local ZWSP already decides removal; the opposite Unknown side is
    // immaterial. Test both directions of that independent decisive evidence.
    for (value, at, expected) in [
        ("\u{200b}\n中", 2, "\u{200b}中"),
        ("中\n\u{200b}", 1, "中\u{200b}"),
    ] {
        let processed =
            process_contextual_source(split(value, at, WhiteSpace::Collapse, None)).unwrap();
        assert_eq!(processed.processed_text(), &Text::from(expected));
        assert_eq!(
            deleted(&processed),
            [(
                Range::new(1, 2),
                crate::processing::DeletionCause::ContextualSegmentBreak
            )]
        );
    }
}

fn before_trim(relationship: Option<ParagraphConnectionKind>) -> ContextualSourceInput {
    let mut request = split(" A", 1, WhiteSpace::Collapse, relationship);
    request.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim {
            discard_before: true,
            ..WhiteSpaceTrim::default()
        },
    });
    // The optional real source connection/barrier is in the preceding raw
    // paragraph, BEFORE target-scope entry. It is not invented at the marker.
    let after_enter = request
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::EnterParagraph(1)))
        .unwrap()
        + 1;
    request
        .events
        .insert(after_enter, SourceProcessingEvent::EnterScope(1));
    let before_exit = request
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::ExitParagraph(1)))
        .unwrap();
    request
        .events
        .insert(before_exit, SourceProcessingEvent::ExitScope(1));
    request
}

#[test]
fn actual_discard_before_scope_demands_its_exterior_source_relationship() {
    // Captured WhiteSpaceTrim runs before Phase I; its genuine exterior
    // target cannot borrow continuity merely from a raw storage marker.
    missing(before_trim(None), 0, 1, SourceContinuityOperation::Trimming);
    for (kind, expected, trimmed) in [
        (ParagraphConnectionKind::Continuous, "A", true),
        (ParagraphConnectionKind::Disconnected, " A", false),
    ] {
        let processed = process_contextual_source(before_trim(Some(kind))).unwrap();
        assert_eq!(processed.processed_text(), &Text::from(expected));
        assert_eq!(
            deleted(&processed),
            if trimmed {
                vec![(Range::new(0, 1), crate::processing::DeletionCause::Trim)]
            } else {
                vec![]
            }
        );
    }
}

fn empty_before_trim() -> ContextualSourceInput {
    let mut request = before_trim(None);
    request
        .origins
        .push(super::origin(&request.source, Range::new(1, 1)));
    request.paragraphs.push(SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    });
    request.connections.push(SourceProcessingConnection {
        before_origin: 2,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    });
    request
        .boundary_policies
        .push(super::policy(ParagraphBoundaryDefinition::Connection(0)));
    let following = request
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::EnterParagraph(1)))
        .unwrap();
    drop(request.events.splice(
        following..following,
        [
            SourceProcessingEvent::EnterParagraph(2),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::ExitParagraph(2),
        ],
    ));
    request
}

#[test]
fn transparent_empty_origin_keeps_the_actual_global_missing_trim_edge_identity() {
    // Event order is origin0 SPACE, origin2 empty, origin1 A. Relation2→1
    // exists, while0→2 is Unknown: neither view IDs nor numeric offset1 is
    // the missing relationship's identity.
    missing(
        empty_before_trim(),
        0,
        2,
        SourceContinuityOperation::Trimming,
    );
    for (kind, expected) in [
        (ParagraphConnectionKind::Continuous, "A"),
        (ParagraphConnectionKind::Disconnected, " A"),
    ] {
        let mut request = empty_before_trim();
        request.connections.push(SourceProcessingConnection {
            before_origin: 0,
            after_origin: 2,
            kind,
        });
        if kind == ParagraphConnectionKind::Continuous {
            request
                .boundary_policies
                .push(super::policy(ParagraphBoundaryDefinition::Connection(1)));
        }
        request
            .events
            .insert(3, SourceProcessingEvent::Connection(1));
        let processed = Arc::new(process_contextual_source(request).unwrap());
        assert_eq!(processed.processed_text(), &Text::from(expected));
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(raw.len(), 3);
        assert_eq!(
            raw[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(1, 1)]
        );
        assert_eq!(processed.captured_origin(2).unwrap().index, 6);
    }
}

#[test]
fn actual_surrogate_pair_needs_decoding_continuity_but_preserves_original_units() {
    // Native contract: decode only actual contiguous original pairs; a known
    // barrier keeps opaque units and never repairs them from glyph output.
    let pair = Text::from_code_units(vec![0xd83d, 0xde00]);
    missing(
        split(pair.clone(), 1, WhiteSpace::Preserve, None),
        0,
        1,
        SourceContinuityOperation::Decoding,
    );
    for (kind, ranges) in [
        (ParagraphConnectionKind::Continuous, vec![Range::new(0, 2)]),
        (
            ParagraphConnectionKind::Disconnected,
            vec![Range::new(0, 1), Range::new(1, 2)],
        ),
    ] {
        let processed =
            process_contextual_source(split(pair.clone(), 1, WhiteSpace::Preserve, Some(kind)))
                .unwrap();
        assert_eq!(processed.source().text(), &pair);
        assert_eq!(processed.processed_text(), &pair);
        assert_eq!(character_ranges(&processed), ranges);
        assert!(deleted(&processed).is_empty());
    }
    let local = process_source_whitespace(
        Source::new(pair.clone()),
        Style {
            white_space: WhiteSpace::Preserve,
            ..Style::default()
        },
        DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    assert_eq!(local.processed_text(), &pair);
    assert_eq!(character_ranges(&local), [Range::new(0, 2)]);
    let nonpair = Text::from_code_units(vec![0xdc00, 0xd800]);
    let opaque =
        process_contextual_source(split(nonpair.clone(), 1, WhiteSpace::Preserve, None)).unwrap();
    assert_eq!(opaque.processed_text(), &nonpair);
    assert_eq!(
        character_ranges(&opaque),
        [Range::new(0, 1), Range::new(1, 2)]
    );
}

#[test]
fn a_real_forced_control_decides_the_fold_without_guessing_an_unknown_source_edge() {
    let mut request = split("  ", 1, WhiteSpace::Collapse, None);
    request.controls.push(ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::ForcedBreak,
    });
    request.events.insert(
        3,
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::Before,
        },
    );
    let processed = Arc::new(process_contextual_source(request).unwrap());
    assert_eq!(processed.processed_text(), &Text::from("  "));
    assert!(deleted(&processed).is_empty());
    assert!(processed.captured_connection(0).is_none());
    let control = processed.captured_control(0).unwrap();
    assert_eq!(
        processed
            .captured_scope(0)
            .unwrap()
            .flow_events()
            .unwrap()
            .filter_map(|event| match event {
                CapturedFlowEvent::Control(control) => Some(control),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [control]
    );
}

#[test]
fn two_sided_inner_trim_has_a_definite_union_across_an_unknown_source_seam() {
    // Preserve isolates actual inner trimming from SPACE collapse. Continuing
    // the source trims both from either side; stopping trims the first from
    // the left and the second from the right. The idempotent union is the
    // same two actual Trim dispositions, so Unknown cannot make it unavailable.
    for relationship in [
        None,
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
    ] {
        let mut request = input("  ", WhiteSpace::Preserve, relationship);
        request.scopes.push(SourceProcessingScope::Inline {
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
        request
            .events
            .insert(1, SourceProcessingEvent::EnterScope(1));
        request.events.insert(
            request.events.len() - 1,
            SourceProcessingEvent::ExitScope(1),
        );
        let source = request.source.clone();
        let processed = Arc::new(process_contextual_source(request).unwrap());
        assert_eq!(processed.source(), &source);
        assert_eq!(processed.processed_text(), &Text::from(""));
        assert_eq!(
            deleted(&processed),
            [
                (Range::new(0, 1), crate::processing::DeletionCause::Trim),
                (Range::new(1, 2), crate::processing::DeletionCause::Trim),
            ]
        );
        assert!(processed.captured_joint_flow(0).is_none());
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(raw.len(), 2);
        assert_eq!(
            raw[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 1)]
        );
        assert_eq!(
            raw[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(1, 2)]
        );
        assert!(super::text(raw[0].plaintext_items(Affinity::After)).is_empty());
        assert!(super::text(raw[1].plaintext_items(Affinity::After)).is_empty());
    }
}

#[test]
fn actual_child_block_exit_authorizes_only_its_immediate_exterior_trim_edge() {
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let mut request = split("AX  B", 1, WhiteSpace::Collapse, None);
        request.origins = [
            Range::new(0, 1),
            Range::new(1, 2),
            Range::new(2, 3),
            Range::new(3, 5),
        ]
        .into_iter()
        .map(|range| super::origin(&request.source, range))
        .collect();
        request.paragraphs = (0..4)
            .map(|_| SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            })
            .collect();
        request.scopes.push(SourceProcessingScope::Block {
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim {
                discard_after: true,
                ..WhiteSpaceTrim::default()
            },
            base_style: Style {
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
            used_direction: ParagraphBaseDirection::LeftToRight,
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
        });
        request.events = vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::EnterParagraph(1),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::EnterParagraph(2),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
        ];
        // The target Block's real exit precedes origin2. No parent0→2 source
        // connection is supplied. Only the independent later edge2→3 varies.
        if let Some(kind) = relationship {
            request.connections.push(SourceProcessingConnection {
                before_origin: 2,
                after_origin: 3,
                kind,
            });
            request.events.push(SourceProcessingEvent::Connection(0));
            if kind == ParagraphConnectionKind::Continuous {
                request
                    .boundary_policies
                    .push(super::policy(ParagraphBoundaryDefinition::Connection(0)));
            }
        }
        request.events.extend([
            SourceProcessingEvent::ExitParagraph(2),
            SourceProcessingEvent::EnterParagraph(3),
            SourceProcessingEvent::Origin {
                origin: 3,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(3),
            SourceProcessingEvent::ExitScope(0),
        ]);
        if relationship.is_none() {
            missing(request, 2, 3, SourceContinuityOperation::Trimming);
        } else {
            let source = request.source.clone();
            let processed = Arc::new(process_contextual_source(request).unwrap());
            assert_eq!(processed.source(), &source);
            let (expected, trimmed) = if relationship == Some(ParagraphConnectionKind::Continuous) {
                ("AXB", vec![Range::new(2, 3), Range::new(3, 4)])
            } else {
                ("AX B", vec![Range::new(2, 3)])
            };
            assert_eq!(processed.processed_text(), &Text::from(expected));
            assert_eq!(
                deleted(&processed),
                trimmed
                    .into_iter()
                    .map(|range| (range, crate::processing::DeletionCause::Trim))
                    .collect::<Vec<_>>()
            );
            assert!(processed.captured_joint_flow(0).is_none());
            assert_eq!(
                processed
                    .paragraphs()
                    .map(|view| view.source_range())
                    .collect::<Vec<_>>(),
                [
                    Range::new(0, 1),
                    Range::new(1, 2),
                    Range::new(2, 3),
                    Range::new(3, 5)
                ]
            );
        }
    }
}

#[test]
fn local_preserved_break_decides_space_removal_across_an_unknown_source_edge() {
    // The following preserved break is in the same original occurrence as
    // SPACE, so its removal is established independently of the earlier edge.
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let request = split("\n \n", 1, WhiteSpace::PreserveBreaks, relationship);
        let processed = process_contextual_source(request).unwrap();
        assert_eq!(processed.processed_text(), &Text::from("\n\n"));
        assert_eq!(
            deleted(&processed),
            [(
                Range::new(1, 2),
                crate::processing::DeletionCause::SpaceAroundSegmentBreak
            )]
        );
    }
}

#[test]
fn enclosing_inner_trim_decides_nested_exterior_trim_across_an_unknown_edge() {
    // The enclosing scope removes both actual SPACE occurrences independently
    // of whether the nested scope's exterior trim can traverse their edge.
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let mut request = input("  ", WhiteSpace::Collapse, relationship);
        for trim in [
            WhiteSpaceTrim {
                discard_inner: true,
                ..WhiteSpaceTrim::default()
            },
            WhiteSpaceTrim {
                discard_before: true,
                ..WhiteSpaceTrim::default()
            },
        ] {
            request.scopes.push(SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                style: None,
                boundaries: None,
                trim,
            });
        }
        request
            .events
            .insert(1, SourceProcessingEvent::EnterScope(1));
        let second_paragraph = request
            .events
            .iter()
            .position(|event| matches!(event, SourceProcessingEvent::EnterParagraph(1)))
            .unwrap();
        request
            .events
            .insert(second_paragraph + 1, SourceProcessingEvent::EnterScope(2));
        let second_paragraph_exit = request
            .events
            .iter()
            .position(|event| matches!(event, SourceProcessingEvent::ExitParagraph(1)))
            .unwrap();
        request
            .events
            .insert(second_paragraph_exit, SourceProcessingEvent::ExitScope(2));
        request.events.insert(
            request.events.len() - 1,
            SourceProcessingEvent::ExitScope(1),
        );
        let processed = process_contextual_source(request).unwrap();
        assert_eq!(processed.processed_text(), &Text::from(""));
        assert_eq!(
            deleted(&processed),
            [
                (Range::new(0, 1), crate::processing::DeletionCause::Trim),
                (Range::new(1, 2), crate::processing::DeletionCause::Trim),
            ]
        );
    }
}

#[test]
fn exterior_break_removal_can_change_an_enclosing_blocks_inner_trim_targets() {
    // Removing the sole leading Break before a Block's inner trim can leave
    // preserved SPACE outside its through-the-last-Break target. The same
    // deletion cause does not make those ordered target sets independent.
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let mut request = split(" \nX", 2, WhiteSpace::Collapse, relationship);
        request.source.span(
            Range::new(0, 1),
            Style {
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
        );
        request.origins = [Range::new(0, 2), Range::new(2, 3)]
            .into_iter()
            .map(|range| super::origin(&request.source, range))
            .collect();
        let SourceProcessingScope::Block { trim, .. } = &mut request.scopes[0] else {
            unreachable!()
        };
        trim.discard_inner = true;
        request.scopes.push(SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            style: None,
            boundaries: None,
            trim: WhiteSpaceTrim {
                discard_before: true,
                ..WhiteSpaceTrim::default()
            },
        });
        let enter = request
            .events
            .iter()
            .position(|event| matches!(event, SourceProcessingEvent::EnterParagraph(1)))
            .unwrap();
        request
            .events
            .insert(enter + 1, SourceProcessingEvent::EnterScope(1));
        let exit = request
            .events
            .iter()
            .position(|event| matches!(event, SourceProcessingEvent::ExitParagraph(1)))
            .unwrap();
        request
            .events
            .insert(exit, SourceProcessingEvent::ExitScope(1));
        if let Some(kind) = relationship {
            let processed = process_contextual_source(request).unwrap();
            let (expected, ranges) = if kind == ParagraphConnectionKind::Continuous {
                (" X", vec![Range::new(1, 2)])
            } else {
                ("X", vec![Range::new(0, 1), Range::new(1, 2)])
            };
            assert_eq!(processed.processed_text(), &Text::from(expected));
            assert_eq!(
                deleted(&processed),
                ranges
                    .into_iter()
                    .map(|range| (range, crate::processing::DeletionCause::Trim))
                    .collect::<Vec<_>>()
            );
        } else {
            missing(request, 0, 1, SourceContinuityOperation::Trimming);
        }
    }
}

#[test]
fn pending_exterior_space_trim_requires_independent_covering_trim() {
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let mut request = input("  ", WhiteSpace::Collapse, relationship);
        request.scopes.push(SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            style: None,
            boundaries: None,
            trim: WhiteSpaceTrim {
                discard_before: true,
                ..WhiteSpaceTrim::default()
            },
        });
        let enter = request
            .events
            .iter()
            .position(|event| matches!(event, SourceProcessingEvent::EnterParagraph(1)))
            .unwrap();
        request
            .events
            .insert(enter + 1, SourceProcessingEvent::EnterScope(1));
        let exit = request
            .events
            .iter()
            .position(|event| matches!(event, SourceProcessingEvent::ExitParagraph(1)))
            .unwrap();
        request
            .events
            .insert(exit, SourceProcessingEvent::ExitScope(1));
        if let Some(kind) = relationship {
            let processed = process_contextual_source(request).unwrap();
            if kind == ParagraphConnectionKind::Continuous {
                assert_eq!(processed.processed_text(), &Text::from(" "));
                assert_eq!(
                    deleted(&processed),
                    [(Range::new(0, 1), crate::processing::DeletionCause::Trim)]
                );
            } else {
                assert_eq!(processed.processed_text(), &Text::from("  "));
                assert!(deleted(&processed).is_empty());
            }
        } else {
            missing(request, 0, 1, SourceContinuityOperation::Trimming);
        }
    }
}
