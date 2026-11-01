//! Actual session-bridge work and canonical pending-scope custody.
//! These are real private session observations, not a global-loop execution
//! trace or an allocator probe. Existing source covers the run_session callers.
use super::ComputeSession;
use crate::engine::{compute_root, round_layout};
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::text_layout::scope_alignment::ScopeAlignmentFacts;
use crate::text_layout::{TextControlBinding, TextOriginBinding, TextParagraphBinding};
use crate::*;
use std::sync::Arc;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const WORK: usize = 1_000_000;
// Limit discovery visits one actual normalized row. Payload admission adds
// normalized key1+1, two line descriptors2 and two frame keys2*(1+1).
// These9 units are a conservative law, not observed CPU/equality execution.
const PASS_WORK: usize = 9;

struct Fixture {
    tree: PublicLayoutTreeOf<f64>,
    binding: TextParagraphBinding<u32>,
    scope: text::CapturedScope,
}
impl Fixture {
    fn new(work: usize) -> Self {
        use text::SourceProcessingEvent as E;
        let style = text::Style {
            font: text::Font::new().family("Ahem"),
            size: 10.0,
            line_height: text::LineHeight::Absolute(10.0),
            white_space: text::WhiteSpace::Preserve,
            word_break: text::WordBreak::BreakAll,
            wrap: text::Wrap::Word,
            hyphens: text::Hyphens::None,
            ..text::Style::default()
        };
        let reference = text::ScopeFontReference {
            script: text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            axis: text::FontBaselineAxis::HorizontalY,
            baseline: text::FontBaseline::Alphabetic,
        };
        let mut source = text::Source::identified("XXXX", text::Id::from_u64(1578080), 71);
        source.span(text::Range::new(0, 4), style.clone());
        let captured = Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
                joint_flows: Vec::new(),
                source,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                envelope: text::SourceProcessingEnvelope::Complete,
                scope_alignment_limits: Some(text::ScopeAlignmentLimits::new(
                    10000,
                    100,
                    work,
                    16 << 20,
                )),
                origins: [("X", 1578081, 0, 1), ("XXX", 1578082, 1, 4)]
                    .into_iter()
                    .map(|(raw, id, start, end)| text::ParagraphOrigin {
                        text_node: None,
                        source: text::Source::identified(raw, text::Id::from_u64(id), 71),
                        local: text::Range::new(0, raw.len()),
                        aggregate: text::Range::new(start, end),
                        kind: text::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                connections: vec![text::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: text::ParagraphConnectionKind::Continuous,
                }],
                controls: vec![
                    text::ParagraphControl {
                        at: 1,
                        kind: text::ParagraphControlKind::InlineBoundaryStart,
                    },
                    text::ParagraphControl {
                        at: 4,
                        kind: text::ParagraphControlKind::InlineBoundaryEnd,
                    },
                ],
                replacements: vec![],
                computed_line_boundaries: vec![],
                boundary_policies: vec![text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: text::Wrap::Word,
                    following_wrap: text::Wrap::Word,
                    ancestor_wrap: text::Wrap::Word,
                }],
                scopes: vec![
                    text::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        topology: text::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: style.clone(),
                        used_direction: text::ParagraphBaseDirection::LeftToRight,
                        font_reference: Some(reference),
                    },
                    text::SourceProcessingScope::Inline {
                        horizontal_inner_direction: None,
                        trim: Default::default(),
                        style: Some(style),
                        boundaries: Some(text::SourceProcessingScopeBoundaries {
                            start: 0,
                            end: 1,
                        }),
                        alignment_boundary: Some(text::ScopeAlignmentBoundary::NonInitial),
                        font_reference: Some(reference),
                    },
                ],
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction: text::ParagraphBaseDirection::LeftToRight,
                    // A genuine root font strut8/2 is present on both semantic lines.
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                }],
                events: vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Origin {
                        origin: 0,
                        affinity: text::Affinity::After,
                    },
                    E::Connection(0),
                    E::EnterScope(1),
                    E::Control {
                        control: 0,
                        affinity: text::Affinity::After,
                    },
                    E::Origin {
                        origin: 1,
                        affinity: text::Affinity::After,
                    },
                    E::Control {
                        control: 1,
                        affinity: text::Affinity::After,
                    },
                    E::ExitScope(1),
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let scope = captured.captured_scope(1).unwrap();
        let analyzed = text::AnalyzedSource::from_processed_paragraph(
            captured.paragraphs().next().unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let analyzed =
            crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = system
            .prepare(
                &analyzed,
                text::PreparationOptions::default()
                    .with_baseline_reference(text::FontBaseline::Alphabetic),
            )
            .unwrap();
        let origins = prepared
            .origin_views()
            .enumerate()
            .map(|(index, origin)| TextOriginBinding {
                origin: origin.handle(),
                node: u32::try_from(index + 1).unwrap(),
            })
            .collect();
        let controls = prepared
            .control_views()
            .map(|control| TextControlBinding {
                occurrence: control.handle(),
                node: match control.kind() {
                    text::ParagraphControlKind::InlineBoundaryStart => 20,
                    text::ParagraphControlKind::InlineBoundaryEnd => 21,
                    _ => panic!("the producer declares exactly its two true endpoints"),
                },
            })
            .collect();
        let binding = TextParagraphBinding {
            inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: 0,
            nodes: vec![1, 20, 2, 21],
            origins,
            controls,
            atomics: vec![],
            line_strut: None,
        };
        let mut tree = PublicLayoutTreeOf::<f64>::new()
            .children(0, binding.nodes.iter().copied())
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    inline_baseline_context: InlineBaselineContext::Named(BaselineName::Alphabetic),
                    size: Size::new(PreferredSizeOf::px(20.0), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            );
        for origin in &binding.origins {
            tree = tree.children(origin.node, []).input(
                origin.node,
                LayoutInputOf::inline_text(TextParticipation::new(origin.origin.clone())),
            );
        }
        for control in &binding.controls {
            let kind = if control.node == 20 {
                InlineBoundaryKind::Start
            } else {
                InlineBoundaryKind::End
            };
            tree = tree.children(control.node, []).input(
                control.node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        kind,
                        InlineMetricsOf::try_new(
                            InlineBaselineContext::Named(BaselineName::Alphabetic),
                            0.0,
                            0.0,
                        )
                        .unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                ),
            );
        }
        Self {
            tree,
            binding,
            scope,
        }
    }
    fn flow(&self) -> LayoutFlowInput<'_, PublicLayoutTreeOf<f64>> {
        LayoutFlowInput::try_new(
            &self.tree,
            0,
            vec![self.binding.clone()],
            WrappingInput::default(),
            vec![TextScopeAlignmentInputOf {
                scope: self.scope.clone(),
                alignment: VerticalAlignOf::Shift(BaselineOffsetOf::try_new(3.0).unwrap()),
            }],
        )
        .unwrap()
    }
}
fn request() -> LayoutRootRequestOf<f64> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(20.0),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}
fn stage(session: &mut ComputeSession<'_, PublicLayoutTreeOf<f64>>) {
    compute_root(session, 0, request().available()).unwrap();
    round_layout(session, 0).unwrap();
}
fn publication(batch: &CompletedLayoutBatchOf<u32, f64>, fixture: &Fixture) {
    let layout = batch
        .placed_text()
        .iter()
        .find(|entry| entry.container() == 0)
        .unwrap()
        .layout();
    assert_eq!(
        fixture
            .binding
            .prepared
            .origin_views()
            .map(|origin| (
                origin.source().id(),
                origin.source().revision(),
                origin.local_range(),
                origin.aggregate_range()
            ))
            .collect::<Vec<_>>(),
        [
            (
                Some(text::Id::from_u64(1578081)),
                71,
                text::Range::new(0, 1),
                text::Range::new(0, 1)
            ),
            (
                Some(text::Id::from_u64(1578082)),
                71,
                text::Range::new(0, 3),
                text::Range::new(1, 4)
            ),
        ]
    );
    assert_eq!(layout.source().text().code_units(), [88, 88, 88, 88]);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| (line.range(), line.baseline(), line.bounds().size.height))
            .collect::<Vec<_>>(),
        [
            (text::Range::new(0, 2), 11.0, 13.0),
            (text::Range::new(2, 4), 24.0, 13.0)
        ]
    );
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
    }
    let glyphs = layout
        .glyph_runs()
        .iter()
        .flat_map(|run| {
            run.glyphs()
                .iter()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.advance(),
                        run.origin().y + f64::from(glyph.y()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [
            (text::Range::new(0, 1), 58, 10.0, 11.0),
            (text::Range::new(1, 2), 58, 10.0, 8.0),
            (text::Range::new(2, 3), 58, 10.0, 21.0),
            (text::Range::new(3, 4), 58, 10.0, 21.0)
        ]
    );
    let mut seen = [false; 2];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        seen[fragment.line_index()] = true;
        assert_eq!(line.prepared_text(), fixture.binding.prepared);
        assert_eq!(line.scope_allocations().len(), 1);
        assert_eq!(line.scope_allocations()[0].scope, fixture.scope);
        assert_eq!(line.scope_allocations()[0].displacement.get(), 3.0);
        assert_eq!(
            line.scope_allocations()[0].cumulative_displacement.get(),
            3.0
        );
        for allocation in &line.selected_line().request().controls {
            assert!(
                fixture
                    .binding
                    .controls
                    .iter()
                    .any(|original| original.occurrence == allocation.occurrence)
            );
            assert_eq!(
                allocation.occurrence.captured_control().unwrap().scope(),
                fixture.scope
            );
            assert_eq!(
                allocation.boundary,
                Some(text::InlineBoundaryAllocation::CONTRIBUTION_ZERO)
            );
            assert_eq!(
                allocation.strut,
                Some(text::ControlStrut {
                    baseline: 0.0,
                    line_extent: 0.0,
                    alignment: text::LineBoxAlignment::Baseline
                })
            );
        }
    }
    assert_eq!(seen, [true, true]);
}

#[test]
fn pending_scope_descriptor_custody_survives_clones_and_retires_with_the_final_outer_owner() {
    let fixture = Fixture::new(WORK);
    let flow = fixture.flow();
    let prior = crate::engine::compute_layout_with_flow(&flow, 0, request()).unwrap();
    publication(&prior, &fixture);
    let usage = fixture.binding.prepared.scope_alignment_usage();
    let mut session = ComputeSession::with_flow(&flow, vec![]);
    stage(&mut session);
    let state = session.text.as_ref().unwrap();
    assert_eq!(state.pending.len(), 1);
    assert_eq!(state.pending[0].entries.len(), 2);
    let bases = state.pending[0].scopes.clone();
    let proofs = bases.iter(2).collect::<Vec<_>>();
    assert!(proofs.iter().all(|proof| proof.comparison_rows() == 1));
    let resources = state.scope_resources.clone();
    // Remove the actual session's pending owner before establishing this local
    // lifetime boundary. Actual inner proof aliases stay pinned independently.
    drop(session);
    let held = ScopeAlignmentFacts::retention(&resources);
    let alias = bases.clone();
    assert_eq!(ScopeAlignmentFacts::retention(&resources), held);
    drop(bases);
    assert_eq!(ScopeAlignmentFacts::retention(&resources), held);
    assert!(
        alias
            .iter(2)
            .zip(&proofs)
            .all(|(actual, expected)| actual == *expected)
    );
    drop(alias);
    let outer_retired = ScopeAlignmentFacts::retention(&resources);
    assert_ne!(
        outer_retired, held,
        "the outer descriptor buffer's own lease must retire"
    );
    assert!(proofs.iter().all(|proof| proof.comparison_rows() == 1));
    drop(proofs);
    assert_ne!(ScopeAlignmentFacts::retention(&resources), outer_retired);
    assert_eq!(fixture.binding.prepared.scope_alignment_usage(), usage);
    publication(&prior, &fixture);
}

fn global_bridge(work: usize) {
    let fixture = Fixture::new(work);
    let flow = fixture.flow();
    let prior = crate::engine::compute_layout_with_flow(&flow, 0, request()).unwrap();
    publication(&prior, &fixture);
    let usage = fixture.binding.prepared.scope_alignment_usage();
    let mut producer = ComputeSession::with_flow(&flow, vec![]);
    stage(&mut producer);
    let frames = producer.canonical_frames(0).unwrap();
    let basis = producer.text_pass_basis(0, frames).unwrap();
    assert_eq!(basis.scope_alignments.rows().len(), 1);
    assert_eq!(basis.lines.len(), 2);
    assert!(
        producer.text.as_ref().unwrap().pending[0]
            .scopes
            .iter(2)
            .all(|proof| proof.comparison_rows() == 1)
    );
    let previous = basis.clone();
    // A fresh real same-flow session starts a genuine work ledger at0. Its
    // actual private global bridge consumes the real native pass products.
    let session = ComputeSession::with_flow(&flow, vec![]);
    let comparisons = WORK / PASS_WORK + usize::from(work > WORK);
    for _ in 0..comparisons {
        session
            .admit_text_pass_comparison(0, Some(&basis), Some(&previous))
            .unwrap();
    }
    assert_eq!(basis, previous);
    if work == WORK {
        for _ in 0..2 {
            let error = session
                .admit_text_pass_comparison(0, Some(&basis), Some(&previous))
                .unwrap_err();
            assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
            assert_eq!(error.operation(), LayoutOperation::TextPlacement);
            let LayoutErrorKindOf::Text(error) = error.kind() else {
                panic!("global scope work must remain a Text resource refusal")
            };
            assert_eq!(error.code, text::ErrorCode::ScopeAlignmentRefused);
            let Some(text::ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
                panic!("missing actual global scope resource detail")
            };
            let text::ScopeAlignmentError::Limit {
                kind,
                required,
                limit,
            } = *reason
            else {
                panic!("global refusal must not become a feedback cycle or miss")
            };
            assert_eq!(kind, text::ScopeAlignmentLimitKind::StructuralWork);
            assert_eq!(limit, WORK);
            assert!(required > limit && required <= limit + PASS_WORK);
            // Discovery may legitimately spend before a later refusal. Do not
            // infer whole-bridge rollback or identical retry diagnostics.
        }
    } else {
        assert_eq!(work, WORK + PASS_WORK);
    }
    assert_eq!(basis, previous);
    assert_eq!(fixture.binding.prepared.scope_alignment_usage(), usage);
    publication(&prior, &fixture);
}

#[test]
fn genuine_global_pass_bridge_admits_scope_payloads_and_propagates_typed_work_refusal() {
    global_bridge(WORK);
    global_bridge(WORK + PASS_WORK);
}
