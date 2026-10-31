//! Combination-only OOF fences must not restart exterior generic SPACE collapse.
use crate::processing::{
    DeletionCause,
    lexical_inventory::{
        LexicalCharacterRole, LexicalContext, LexicalDisposition, LexicalRecordKind, LexicalState,
    },
};
use crate::protocol::paragraph::SemanticAnchorItem;
use crate::transform::relation::{ComponentOwner, RelationEvent};
use crate::*;
use std::sync::Arc;

fn capture(mode: TextCombineUpright, with_oof: bool) -> Arc<ProcessedSource> {
    // New minimal topology: only A belongs to the Inline. Both following
    // SPACE originals and final Z genuinely belong to Root None, with OOF
    // optionally between the SPACE occurrences.
    let mut input = super::input("A  Z", WhiteSpace::Collapse);
    let SourceProcessingScope::Block {
        topology,
        base_style,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual root Block");
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    base_style.text_combine_upright = TextCombineUpright::None;
    base_style.text_transform = TextTransform::none();
    let inside = Style {
        text_combine_upright: mode,
        ..base_style.clone()
    };
    input.source.span(Range::new(0, 1), inside.clone());
    input.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(inside),
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
    });
    if with_oof {
        input.source.inline_box(InlineBox::new(
            Id::from_u64(784_995_001),
            InlineBoxKind::OutOfFlow,
            2,
            Size::new(5.0, 10.0),
        ));
    }
    input.origins = (0..4)
        .map(|at| super::origin(&input.source, Range::new(at, at + 1)))
        .collect();
    input.connections = (0..3)
        .map(|before| SourceProcessingConnection {
            before_origin: before,
            after_origin: before + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    input.boundary_policies = (0..3)
        .map(|edge| super::policy(ParagraphBoundaryDefinition::Connection(edge)))
        .collect();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
    ];
    if with_oof {
        input.events.push(SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::After,
        });
    }
    input.events.extend([
        SourceProcessingEvent::Connection(1),
        SourceProcessingEvent::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Connection(2),
        SourceProcessingEvent::Origin {
            origin: 3,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    Arc::new(process_contextual_source(input).unwrap())
}

fn phase_one_premises(
    processed: &Arc<ProcessedSource>,
    mode: TextCombineUpright,
    with_oof: bool,
) -> usize {
    assert_eq!(processed.processed_text(), &Text::from("A Z"));
    let root = processed.captured_scope(0).unwrap();
    let inline = processed.captured_scope(1).unwrap();
    assert_eq!(inline.parent(), Some(root.clone()));
    assert_eq!(inline.style().unwrap().text_combine_upright, mode);
    assert_eq!(inline.style().unwrap().white_space, WhiteSpace::Collapse);
    assert_eq!(
        root.style().unwrap().text_combine_upright,
        TextCombineUpright::None
    );
    assert_eq!(root.style().unwrap().white_space, WhiteSpace::Collapse);
    assert!(inline.enter_position() < processed.captured_origin(0).unwrap());
    assert!(processed.captured_origin(0).unwrap() < inline.exit_position());
    assert!(inline.exit_position() < processed.captured_origin(1).unwrap());
    for edge in 0..3 {
        assert_eq!(
            processed.captured_connection(edge).unwrap().kind(),
            ParagraphConnectionKind::Continuous
        );
    }
    let raw = processed.paragraphs().next().unwrap();
    let topology = raw.data().scope_topology.as_ref().unwrap();
    for origin in [1, 2, 3] {
        let order = processed.captured_origin(origin).unwrap().index;
        let scope = topology.capture.events[order].before.unwrap();
        assert_eq!(processed.captured_scope(scope).unwrap(), root);
    }
    assert!(processed.captured_origin(2).unwrap() < processed.captured_origin(3).unwrap());
    assert!(processed.captured_origin(4).is_none());
    if with_oof {
        let atomic = processed.captured_atomic(0).unwrap();
        assert!(processed.captured_origin(1).unwrap() < atomic);
        assert!(atomic < processed.captured_origin(2).unwrap());
        let scope = topology.capture.events[atomic.index].before.unwrap();
        assert_eq!(processed.captured_scope(scope).unwrap(), root);
        assert_eq!(
            processed.source().boxes()[0].kind(),
            InlineBoxKind::OutOfFlow
        );
        assert_eq!(processed.source().boxes()[0].index(), 2);
    } else {
        assert!(processed.captured_atomic(0).is_none());
    }
    let LexicalState::Executed { context, records } = &processed.lexical else {
        panic!("actual executed Phase I");
    };
    let LexicalContext::Captured(capture) = context else {
        panic!("actual captured lexical owner");
    };
    assert!(Arc::ptr_eq(capture, &topology.capture));
    let first = records
        .iter()
        .filter(|record| {
            matches!(record.kind,
        LexicalRecordKind::Character { source, role: LexicalCharacterRole::Space, .. }
        if source == Range::new(1, 2))
        })
        .collect::<Vec<_>>();
    let [first] = first.as_slice() else {
        panic!("one actual first SPACE occurrence");
    };
    assert!(matches!(first.kind, LexicalRecordKind::Character {
        mode: WhiteSpace::Collapse, disposition: LexicalDisposition::Emitted { processed }, ..
    } if processed == Range::new(1, 2)));
    let final_z = records
        .iter()
        .filter(|record| {
            matches!(record.kind,
        LexicalRecordKind::Character { source, role: LexicalCharacterRole::Text, .. }
        if source == Range::new(3, 4))
        })
        .collect::<Vec<_>>();
    let [final_z] = final_z.as_slice() else {
        panic!("one genuine exterior Z occurrence");
    };
    assert!(matches!(final_z.kind, LexicalRecordKind::Character {
        mode: WhiteSpace::Collapse, disposition: LexicalDisposition::Emitted { processed }, ..
    } if processed == Range::new(2, 3)));
    let deletion = processed
        .lexical
        .deletion(Range::new(2, 3), DeletionCause::CollapsedSpace);
    let collapsed = deletion
        .indexed_occurrences(&processed.lexical)
        .collect::<Vec<_>>();
    let [(index, second)] = collapsed.as_slice() else {
        panic!("one genuine collapsed-zero second SPACE");
    };
    assert!(matches!(second.kind, LexicalRecordKind::Character {
        source, mode: WhiteSpace::Collapse, role: LexicalCharacterRole::Space,
        disposition: LexicalDisposition::Deleted { processed: 2, cause: DeletionCause::CollapsedSpace },
    } if source == Range::new(2, 3)));
    assert_eq!(
        deletion.original_range(&processed.lexical),
        Range::new(2, 3)
    );
    *index
}

#[test]
fn an_a_cell_does_not_revive_a_second_exterior_space_across_transparent_oof() {
    // None+OOF qualifies generic continuity. All without OOF qualifies the real
    // Cell/exterior partition. Only the final case composes both mechanisms.
    for (mode, with_oof) in [
        (TextCombineUpright::None, true),
        (TextCombineUpright::All, false),
        (TextCombineUpright::All, true),
    ] {
        let processed = capture(mode, with_oof);
        let zero_record = phase_one_premises(&processed, mode, with_oof);
        let domain = AnalyzedDomain::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(Arc::ptr_eq(domain.processed_source(), &processed));
        assert_eq!(domain.outputs().len(), 1);
        let output = domain.outputs().next().unwrap();
        assert_eq!(output.source(), processed.source());
        assert_eq!(
            output.combined_cells().len(),
            usize::from(mode == TextCombineUpright::All)
        );
        if mode == TextCombineUpright::All {
            let cell = output.combined_cells().next().unwrap();
            assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
            assert_eq!(
                cell.parent_scope(),
                Some(processed.captured_scope(0).unwrap())
            );
            assert_eq!(cell.selection_unit_count(), 1);
            assert_eq!(cell.used_unit_count(), 1);
            assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41]);
            let pieces = cell.source_pieces().collect::<Vec<_>>();
            let [piece] = pieces.as_slice() else {
                panic!("A alone belongs to the Cell");
            };
            assert_eq!(piece.local(), Range::new(0, 1));
            assert_eq!(
                piece.origin_position(),
                processed.captured_origin(0).unwrap()
            );
        }
        // Phase I retained the FIRST exterior SPACE. Neither the Cell boundary
        // before that SPACE nor the later transparent OOF can justify reviving
        // the second one. Z makes this an interior SPACE, independent of later
        // trailing-line-edge trimming. This source scalar+occurrence oracle precedes any
        // line-edge Phase II, font query, shaping or geometry measurement.
        assert_eq!(
            output
                .inner
                .projection
                .atoms
                .iter()
                .map(|atom| (atom.code_point, atom.source))
                .collect::<Vec<_>>(),
            [
                (0x41, Range::new(0, 1)),
                (0x20, Range::new(1, 2)),
                (0x5a, Range::new(3, 4))
            ]
        );
        let component = output.inner.transform.component_for_atom(1);
        let ComponentOwner::Original(row) = component.owner else {
            panic!("the visible SPACE keeps original custody");
        };
        let pieces = output.inner.transform.original_pieces(row);
        let [piece] = pieces else {
            panic!("one actual exterior SPACE original");
        };
        assert_eq!(
            (piece.local, piece.aggregate),
            (Range::new(1, 2), Range::new(1, 2))
        );
        assert_eq!(
            piece.origin.captured_event(),
            Some(processed.captured_origin(1).unwrap().index)
        );
        let z_component = output.inner.transform.component_for_atom(2);
        let ComponentOwner::Original(z_row) = z_component.owner else {
            panic!("the exterior Z keeps original custody");
        };
        let z_pieces = output.inner.transform.original_pieces(z_row);
        let [z_piece] = z_pieces else {
            panic!("one actual exterior Z original");
        };
        assert_eq!(
            (z_piece.local, z_piece.aggregate),
            (Range::new(3, 4), Range::new(3, 4))
        );
        assert_eq!(
            z_piece.origin.captured_event(),
            Some(processed.captured_origin(3).unwrap().index)
        );
        assert!(
            output.inner.transform.events().iter().any(|event| {
                if let RelationEvent::PhaseIDeleted { lexical, .. } = event {
                    lexical
                        .indexed_occurrences(&processed.lexical)
                        .any(|(index, _)| index == zero_record)
                } else {
                    false
                }
            }),
            "the second SPACE remains its genuine collapsed-zero occurrence"
        );
        if with_oof {
            let anchors = output
                .inner
                .paragraph
                .anchor_order
                .iter()
                .filter(|anchor| matches!(anchor.item, SemanticAnchorItem::Atomic(_)))
                .collect::<Vec<_>>();
            let [anchor] = anchors.as_slice() else {
                panic!("actual OOF remains source-owned");
            };
            assert_eq!(anchor.affinity, Affinity::After);
            let SemanticAnchorItem::Atomic(atomic) = anchor.item else {
                unreachable!();
            };
            assert_eq!(
                output.inner.processed.source().boxes()[atomic].id(),
                Id::from_u64(784_995_001)
            );
            assert_eq!(
                output
                    .inner
                    .paragraph
                    .scope_topology
                    .as_ref()
                    .unwrap()
                    .capture
                    .atomic_orders[atomic],
                processed.captured_atomic(0).unwrap().index
            );
        }
        // Used replay must not rewrite the already executed immutable Phase I.
        assert_eq!(phase_one_premises(&processed, mode, with_oof), zero_record);
    }
}
