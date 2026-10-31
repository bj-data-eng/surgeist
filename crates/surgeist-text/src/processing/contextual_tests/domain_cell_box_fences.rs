//! Actual box fences preserve canonical Source units before native preparation.
//! Empty inline lookaround, exterior OOF identity and uninterrupted force
//! suppression retain distinct owners.
use super::*;

#[test]
fn empty_inline_box_prevents_combination_while_uninterrupted_text_suppresses_its_force() {
    // Reuse the actual captured factories, not a glyph/geometry surrogate:
    // Control is A, genuine ForcedBreak, B in one scope;
    // ComputedScopeSide is A, genuine empty Inline, B with that box's Before
    // computed force. Both retain the original two occurrences and cut1.
    for force in [Force::Control, Force::ComputedScopeSide] {
        for mode in [TextCombineUpright::None, TextCombineUpright::All] {
            let processed = capture(mode, force);
            if matches!(force, Force::ComputedScopeSide) {
                let root = processed.captured_scope(0).unwrap();
                let empty = processed.captured_scope(1).unwrap();
                assert_eq!(empty.role(), ScopeRole::Inline);
                assert_eq!(empty.parent(), Some(root));
                // There are exactly two real originals, both outside the box.
                // Its emptiness comes from capture chronology, never font ink.
                assert!(processed.captured_origin(0).unwrap() < empty.enter_position());
                assert!(empty.enter_position() < empty.exit_position());
                assert!(empty.exit_position() < processed.captured_origin(1).unwrap());
                assert!(processed.captured_origin(2).is_none());
            } else {
                assert!(processed.captured_scope(1).is_none());
            }
            let domain = AnalyzedDomain::from_processed_paragraph(
                processed.paragraphs().next().unwrap(),
                AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert!(Arc::ptr_eq(domain.processed_source(), &processed));
            assert_eq!(domain.outputs().len(), 1);
            let output = domain.outputs().next().unwrap();
            assert_eq!(output.source(), processed.source());
            let force_position = retained_force(output, &processed, force);
            let end = output.inner.logical.events.len();
            let combines = mode == TextCombineUpright::All && matches!(force, Force::Control);
            // WM4 9.1.1: no Cell crosses a genuine box boundary. With inherited
            // All on both sides, lookaround rejects both partial candidates;
            // skipping empty events during lookaround is not crossing permission.
            // WM4 9.1.2 separately ignores a force inside genuinely plain text.
            assert_eq!(output.combined_cells().len(), usize::from(combines));
            if combines {
                let cell = output.combined_cells().next().unwrap();
                assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
                assert_eq!(cell.selection_unit_count(), 2);
                assert_eq!(cell.used_unit_count(), 2);
                assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x42]);
                assert_eq!(
                    cell.inner_bidi_levels()
                        .map(BidiLevel::get)
                        .collect::<Vec<_>>(),
                    [0, 0]
                );
                assert!(!output.inner.analysis.line_units.permits_edge(1));
                assert!(
                    output
                        .inner
                        .logical
                        .relations
                        .iter()
                        .all(|relation| !relation.before_hard_separator)
                );
                assert_eq!(
                    output.inner.direction.query_in(0, 0, end),
                    Some((Some(0), true))
                );
            } else {
                // Both None controls and the rejected All sequence retain their
                // ordinary force effect. The genuine cause was checked above.
                assert!(
                    output
                        .inner
                        .logical
                        .relations
                        .iter()
                        .any(|relation| relation.before_hard_separator)
                );
                assert_eq!(output.inner.direction.query_in(0, 0, end), None);
                assert_eq!(
                    output.inner.direction.query_in(0, force_position + 1, end),
                    Some((Some(0), true))
                );
            }
        }
    }
}

fn out_of_flow_capture(
    text: &str,
    mode: TextCombineUpright,
    affinity: Affinity,
) -> Arc<ProcessedSource> {
    assert_eq!(text.encode_utf16().count(), 2);
    let mut input = super::super::input(text, WhiteSpace::Preserve);
    let SourceProcessingScope::Block {
        topology,
        base_style,
        used_direction,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("genuine root Block");
    };
    *topology = SourceProcessingTopology::Complete;
    *used_direction = ParagraphBaseDirection::LeftToRight;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    base_style.text_combine_upright = mode;
    base_style.text_transform = TextTransform::none();
    let own = base_style.clone();
    input.source.span(Range::new(0, 2), own);
    input.source.inline_box(InlineBox::new(
        Id::from_u64(784_994_001),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(5.0, 10.0),
    ));
    input.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| super::super::origin(&input.source, range))
        .collect();
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![super::super::policy(
        ParagraphBoundaryDefinition::Connection(0),
    )];
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity,
        },
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    Arc::new(process_contextual_source(input).unwrap())
}

#[test]
fn out_of_flow_occurrence_fences_combination_without_becoming_text_or_inline_only_lookaround() {
    // Unlike an inline-only boundary, a genuine OOF separator makes each side
    // independently eligible. All therefore creates [A Cell], OOF, [B Cell],
    // never one A[OOF]B Cell and never an invented provider/object scalar.
    for affinity in [Affinity::Before, Affinity::After] {
        for mode in [TextCombineUpright::None, TextCombineUpright::All] {
            let processed = out_of_flow_capture("AB", mode, affinity);
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
                output
                    .inner
                    .projection
                    .atoms
                    .iter()
                    .map(|atom| atom.code_point)
                    .collect::<Vec<_>>(),
                [65, 66]
            );
            let relation = preserved_oof(output, &processed, affinity);
            let expected = if mode == TextCombineUpright::All {
                2
            } else {
                0
            };
            assert_eq!(output.combined_cells().len(), expected);
            if mode == TextCombineUpright::All {
                for (at, cell) in output.combined_cells().enumerate() {
                    assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
                    assert_eq!(cell.selection_unit_count(), 1);
                    assert_eq!(cell.used_unit_count(), 1);
                    assert_eq!(
                        cell.used_code_points().collect::<Vec<_>>(),
                        [if at == 0 { 65 } else { 66 }]
                    );
                    let pieces = cell.source_pieces().collect::<Vec<_>>();
                    let [piece] = pieces.as_slice() else {
                        panic!("one original per side");
                    };
                    assert_eq!(piece.local(), Range::new(at, at + 1));
                    assert_eq!(
                        piece.origin_position(),
                        processed.captured_origin(at).unwrap()
                    );
                    assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
                    assert!(piece.is_used());
                }
                // Actual emitted binding incidence, not a source envelope:
                // the exterior OOF occurrence belongs to neither Cell.
                for binding in &output.inner.combination.as_ref().unwrap().bindings {
                    assert!(!binding.events.contains(&relation));
                }
            }
        }
    }
}

fn preserved_oof(
    output: &AnalyzedSource,
    processed: &ProcessedSource,
    affinity: Affinity,
) -> usize {
    let root = processed.captured_scope(0).unwrap();
    let occurrence = processed.captured_atomic(0).unwrap();
    assert!(processed.captured_origin(0).unwrap() < occurrence);
    assert!(occurrence < processed.captured_origin(1).unwrap());
    assert!(processed.captured_atomic(1).is_none());
    let observed = root
        .flow_events()
        .unwrap()
        .filter_map(|event| {
            if let CapturedFlowEvent::Atomic {
                paragraph,
                position,
            } = event
            {
                Some((paragraph, position))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        [(processed.captured_paragraph(0).unwrap(), occurrence.clone())]
    );
    let anchors = output
        .inner
        .paragraph
        .anchor_order
        .iter()
        .enumerate()
        .filter(|(_, anchor)| matches!(anchor.item, SemanticAnchorItem::Atomic(_)))
        .collect::<Vec<_>>();
    let [(order, anchor)] = anchors.as_slice() else {
        panic!("one genuine atomic occurrence");
    };
    assert_eq!(anchor.affinity, affinity);
    let SemanticAnchorItem::Atomic(atomic) = anchor.item else {
        unreachable!();
    };
    let original = output.inner.processed.source().boxes()[atomic];
    assert_eq!(original.id(), Id::from_u64(784_994_001));
    assert_eq!(original.kind(), InlineBoxKind::OutOfFlow);
    assert_eq!(original.index(), 1);
    let topology = output.inner.paragraph.scope_topology.as_ref().unwrap();
    assert_eq!(topology.capture.atomic_orders[atomic], occurrence.index);
    let actual_scope = topology.capture.events[occurrence.index].before.unwrap();
    assert_eq!(processed.captured_scope(actual_scope).unwrap(), root);
    output.inner.transform.events().iter().position(|event| {
        matches!(*event, crate::transform::relation::RelationEvent::Anchor { order: actual } if actual == *order)
    }).expect("the actual OOF remains in the canonical relation")
}

#[test]
fn an_egc_straddling_out_of_flow_box_is_not_split_into_partially_combined_units() {
    // A + Extend is one canonical EGC (GB9). OOF remains transparent to the
    // generic transformed scalar/EGC context, but its genuine box interrupts
    // combination ownership. Reject the entire crossing unit; do not introduce
    // an artificial grapheme cut or select A/mark as two separate Cells.
    for affinity in [Affinity::Before, Affinity::After] {
        for mode in [TextCombineUpright::None, TextCombineUpright::All] {
            let processed = out_of_flow_capture("A\u{301}", mode, affinity);
            let domain = AnalyzedDomain::from_processed_paragraph(
                processed.paragraphs().next().unwrap(),
                AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert!(Arc::ptr_eq(domain.processed_source(), &processed));
            assert_eq!(domain.outputs().len(), 1);
            let output = domain.outputs().next().unwrap();
            assert_eq!(
                output
                    .inner
                    .projection
                    .atoms
                    .iter()
                    .map(|atom| atom.code_point)
                    .collect::<Vec<_>>(),
                [0x41, 0x301]
            );
            assert_eq!(
                output.inner.projection.graphemes.len(),
                2,
                "one genuine EGC plus its two edge cuts"
            );
            let relation = preserved_oof(output, &processed, affinity);
            assert_eq!(output.combined_cells().len(), 0);
            if let Some(combination) = &output.inner.combination {
                assert!(
                    combination
                        .bindings
                        .iter()
                        .all(|binding| !binding.events.contains(&relation))
                );
            }
        }
    }
}
