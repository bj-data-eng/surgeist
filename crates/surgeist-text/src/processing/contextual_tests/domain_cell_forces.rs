//! Source-cell suppression of real interior forces. These are font-independent
//! functional qualifications of the completed-cell/source analysis front.
//! The adopted source-stage model's inner-edge rule suppresses forced effects
//! after selection while preserving the genuine occurrence/cause and source.

use crate::engine::analysis::wrapping::Event;
use crate::protocol::paragraph::SemanticAnchorItem;
use crate::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
enum Force {
    Control,
    ComputedScopeSide,
}

fn capture(mode: TextCombineUpright, force: Force) -> Arc<ProcessedSource> {
    let mut input = super::input("AB", WhiteSpace::Preserve);
    let SourceProcessingScope::Block {
        topology,
        base_style,
        used_direction,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual root Block")
    };
    *topology = SourceProcessingTopology::Complete;
    *used_direction = ParagraphBaseDirection::RightToLeft;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    base_style.text_combine_upright = mode;
    base_style.text_transform = TextTransform::none();
    input.paragraphs[0].base_direction = ParagraphBaseDirection::RightToLeft;
    input.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![super::policy(ParagraphBoundaryDefinition::Connection(0))];
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
    ];
    match force {
        Force::Control => {
            input.controls = vec![ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::ForcedBreak,
            }];
            input.events.push(SourceProcessingEvent::Control {
                control: 0,
                affinity: Affinity::Before,
            });
        }
        Force::ComputedScopeSide => {
            let mut style = super::style(WhiteSpace::Preserve, 10.0);
            style.text_combine_upright = mode;
            // This is a genuinely empty inline box at the actual A/B cut.
            // No B character is moved inside it to force a box-crossing cell.
            input.scopes.push(SourceProcessingScope::Inline {
                trim: WhiteSpaceTrim::default(),
                style: Some(style),
                boundaries: None,
                alignment_boundary: None,
                font_reference: None,
                horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
            });
            input.events.extend([
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::ExitScope(1),
            ]);
            input.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: SourceBoxSide::Before,
            }];
        }
    }
    input.events.extend([
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("AB"));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        [0x41, 0x42]
    );
    assert!(processed.captured_joint_flow(0).is_none());
    let connection = processed.captured_connection(0).unwrap();
    assert_eq!(connection.kind(), ParagraphConnectionKind::Continuous);
    let raw = processed.paragraphs().next().unwrap();
    // This is a local connection: its genuine policy remains paragraph-owned.
    assert_eq!(raw.data().boundary_policies.len(), 1);
    assert_eq!(
        raw.data().boundary_policies[0].definition,
        ParagraphBoundaryDefinition::Connection(0)
    );
    assert_eq!(
        raw.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    processed
}

fn retained_force(output: &AnalyzedSource, processed: &ProcessedSource, force: Force) -> usize {
    let paragraph = &output.inner.paragraph;
    let position = output
        .inner
        .logical
        .events
        .iter()
        .position(|event| match event.kind {
            Event::Anchor(anchor) => match (force, anchor.item) {
                (Force::Control, SemanticAnchorItem::Control(control)) => {
                    let captured = processed.captured_control(0).unwrap();
                    assert_eq!(captured.kind(), &ParagraphControlKind::ForcedBreak);
                    assert_eq!(captured.source_position().get(), 1);
                    assert_eq!(captured.scope(), processed.captured_scope(0).unwrap());
                    assert_eq!(
                        captured.paragraph(),
                        processed.captured_paragraph(0).unwrap()
                    );
                    assert_eq!(
                        paragraph.controls[control].kind,
                        ParagraphControlKind::ForcedBreak
                    );
                    assert_eq!(paragraph.controls[control].at, 1);
                    assert_eq!(
                        paragraph.scope_topology.as_ref().unwrap().control_orders[control],
                        captured.event_position().index
                    );
                    true
                }
                (Force::ComputedScopeSide, SemanticAnchorItem::ComputedLineBoundary(boundary)) => {
                    let captured = processed.captured_computed_line_boundary(0).unwrap();
                    assert_eq!(
                        captured.target(),
                        CapturedComputedLineBoundaryTarget::Scope {
                            scope: processed.captured_scope(1).unwrap(),
                            side: SourceBoxSide::Before,
                        }
                    );
                    assert_eq!(captured.source_position().get(), 1);
                    assert_eq!(captured.domain(), processed.captured_scope(0).unwrap());
                    assert_eq!(
                        captured.paragraph(),
                        processed.captured_paragraph(0).unwrap()
                    );
                    assert_eq!(paragraph.computed_boundaries[boundary].at, 1);
                    assert_eq!(
                        paragraph.computed_boundaries[boundary].causes,
                        [captured.index]
                    );
                    true
                }
                _ => false,
            },
            _ => false,
        })
        .expect("the real force remains in the canonical source occurrence stream");
    assert!(position > 0 && position + 1 < output.inner.logical.events.len());
    position
}

fn qualify(force: Force) {
    // None is an ordinary positive control for the SAME genuine force;
    // suppressing an accepted cell must not globally neutralize source forces.
    for mode in [TextCombineUpright::None, TextCombineUpright::All] {
        let processed = capture(mode, force);
        let domain = AnalyzedDomain::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(Arc::ptr_eq(domain.processed_source(), &processed));
        assert_eq!(domain.outputs().len(), 1);
        let output = domain.outputs().next().unwrap();
        assert_eq!(output.source(), processed.source());
        assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
        assert_eq!(output.inner.projection.text, "AB");
        assert_eq!(
            output
                .inner
                .projection
                .atoms
                .iter()
                .map(|atom| (atom.code_point, atom.source))
                .collect::<Vec<_>>(),
            [(0x41, Range::new(0, 1)), (0x42, Range::new(1, 2)),]
        );
        let owner = output.captured_owner().unwrap();
        assert_eq!(
            owner.members().collect::<Vec<_>>(),
            [processed.captured_paragraph(0).unwrap()]
        );
        assert!(owner.joint_flow().is_none());
        let force_position = retained_force(output, &processed, force);
        let end = output.inner.logical.events.len();
        if mode == TextCombineUpright::None {
            assert_eq!(output.combined_cells().len(), 0);
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
            continue;
        }
        assert_eq!(output.combined_cells().len(), 1);
        let cell = output.combined_cells().next().unwrap();
        assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
        assert_eq!(cell.mode(), TextCombineUpright::All);
        assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
        assert_eq!(cell.selection_unit_count(), 2);
        assert_eq!(cell.used_unit_count(), 2);
        assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x42]);
        assert_eq!(
            cell.inner_bidi_levels()
                .map(|level| level.get())
                .collect::<Vec<_>>(),
            [0, 0]
        );
        assert_eq!(cell.outer_bidi_level().get(), 2);
        assert_eq!(
            cell.source_pieces()
                .map(|piece| {
                    assert_eq!(piece.source(), processed.source());
                    assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
                    assert!(piece.is_used());
                    (
                        piece.local(),
                        piece.aggregate(),
                        piece.origin_position(),
                        piece.paragraph(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (
                    Range::new(0, 1),
                    Range::new(0, 1),
                    processed.captured_origin(0).unwrap(),
                    processed.captured_paragraph(0).unwrap()
                ),
                (
                    Range::new(1, 2),
                    Range::new(1, 2),
                    processed.captured_origin(1).unwrap(),
                    processed.captured_paragraph(0).unwrap()
                ),
            ]
        );
        // The one outer upright-L cell cannot contain a native hard separator,
        // a direction-body restart, or an interior line-unit edge. Original
        // force custody above remains independent of these executed effects.
        assert_eq!(
            output.inner.direction.query_in(0, 0, end),
            Some((Some(0), true))
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
        assert_eq!(processed.source().text(), &Text::from("AB"));
    }
}

#[test]
fn a_real_forced_break_inside_an_accepted_all_cell_keeps_custody_without_outer_force() {
    qualify(Force::Control);
}

#[test]
fn a_computed_empty_inline_side_inside_an_accepted_all_cell_keeps_its_cause_without_outer_force() {
    qualify(Force::ComputedScopeSide);
}
