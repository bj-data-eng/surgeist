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
const WORK: usize = 4_194_304;
// Limit discovery visits one actual normalized row. Payload admission adds
// normalized key1+1, two line descriptors2 and two frame keys2*(1+1).
// These9 units are a conservative law, not observed CPU/equality execution.
const PASS_WORK: usize = 9;

struct Fixture {
    tree: PublicLayoutTreeOf<f64>,
    binding: TextParagraphBinding<u32>,
    scope: text::capture::CapturedScope,
}
impl Fixture {
    fn new(work: usize) -> Self {
        use text::capture::SourceProcessingEvent as E;
        let style = text::style::Style {
            font: text::style::Font::new().family("Ahem"),
            size: 10.0,
            line_height: text::style::LineHeight::Absolute(10.0),
            white_space: text::style::WhiteSpace::Preserve,
            word_break: text::style::WordBreak::BreakAll,
            wrap: text::style::Wrap::Word,
            hyphens: text::hyphenation::Hyphens::None,
            ..text::style::Style::default()
        };
        let reference = text::scope_alignment::ScopeFontReference {
            script: text::font::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            axis: text::font::FontBaselineAxis::HorizontalY,
            baseline: text::font::FontBaseline::Alphabetic,
        };
        let mut source =
            text::source::Source::identified("XXXX", text::source::Id::from_u64(1578080), 71);
        source.span(text::source::Range::new(0, 4), style.clone());
        let captured = Arc::new(
            text::capture::process_contextual_source(text::capture::ContextualSourceInput {
                joint_flows: Vec::new(),
                source,
                segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
                envelope: text::capture::SourceProcessingEnvelope::Complete,
                scope_alignment_limits: Some(text::scope_alignment::ScopeAlignmentLimits::new(
                    10000,
                    100,
                    work,
                    16 << 20,
                )),
                origins: [("X", 1578081, 0, 1), ("XXX", 1578082, 1, 4)]
                    .into_iter()
                    .map(|(raw, id, start, end)| text::paragraph::ParagraphOrigin {
                        text_node: None,
                        source: text::source::Source::identified(
                            raw,
                            text::source::Id::from_u64(id),
                            71,
                        ),
                        local: text::source::Range::new(0, raw.len()),
                        aggregate: text::source::Range::new(start, end),
                        kind: text::paragraph::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                connections: vec![text::capture::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: text::paragraph::ParagraphConnectionKind::Continuous,
                }],
                controls: vec![
                    text::paragraph::ParagraphControl {
                        at: 1,
                        kind: text::paragraph::ParagraphControlKind::InlineBoundaryStart,
                    },
                    text::paragraph::ParagraphControl {
                        at: 4,
                        kind: text::paragraph::ParagraphControlKind::InlineBoundaryEnd,
                    },
                ],
                replacements: vec![],
                computed_line_boundaries: vec![],
                boundary_policies: vec![text::paragraph::ParagraphBoundaryPolicy {
                    definition: text::paragraph::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: text::style::Wrap::Word,
                    following_wrap: text::style::Wrap::Word,
                    ancestor_wrap: text::style::Wrap::Word,
                }],
                scopes: vec![
                    text::capture::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        topology: text::capture::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: style.clone(),
                        used_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                        font_reference: Some(reference),
                    },
                    text::capture::SourceProcessingScope::Inline {
                        horizontal_inner_direction: None,
                        trim: Default::default(),
                        style: Some(style),
                        boundaries: Some(text::capture::SourceProcessingScopeBoundaries {
                            start: 0,
                            end: 1,
                        }),
                        alignment_boundary: Some(
                            text::scope_alignment::ScopeAlignmentBoundary::NonInitial,
                        ),
                        font_reference: Some(reference),
                    },
                ],
                paragraphs: vec![text::capture::SourceProcessingParagraph {
                    base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                    // A root font strut8/2 is present on both semantic lines.
                    paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
                }],
                events: vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Origin {
                        origin: 0,
                        affinity: text::layout::Affinity::After,
                    },
                    E::Connection(0),
                    E::EnterScope(1),
                    E::Control {
                        control: 0,
                        affinity: text::layout::Affinity::After,
                    },
                    E::Origin {
                        origin: 1,
                        affinity: text::layout::Affinity::After,
                    },
                    E::Control {
                        control: 1,
                        affinity: text::layout::Affinity::After,
                    },
                    E::ExitScope(1),
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let scope = captured.captured_scope(1).unwrap();
        let analyzed = text::analyzed::AnalyzedSource::from_processed_paragraph(
            captured.paragraphs().next().unwrap(),
            text::analyzed::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let analyzed =
            crate::test_support::text::in_flat_wrapping_context(analyzed, text::style::Wrap::Word);
        let mut system = text::System::default();
        system
            .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
            .unwrap();
        let prepared = system
            .prepare(
                &analyzed,
                text::prepared::PreparationOptions::default()
                    .with_baseline_reference(text::font::FontBaseline::Alphabetic),
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
                    text::paragraph::ParagraphControlKind::InlineBoundaryStart => 20,
                    text::paragraph::ParagraphControlKind::InlineBoundaryEnd => 21,
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
    {
        let containing_flow_axes =
            crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr);
        let requested_axes = Size::new(
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(20.0).unwrap()),
                crate::SizingOperation::Normal,
            ),
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::MaxContent,
            ),
        );
        let mut logical_requests = containing_flow_axes.logical_size(requested_axes);
        // These fixtures perform ordinary block-size layout independently of
        // the explicitly requested inline contribution operation.
        logical_requests.block = logical_requests
            .block
            .with_operation(crate::SizingOperation::Normal);
        logical_requests.inline = logical_requests
            .inline
            .with_operation(crate::SizingOperation::Normal);
        let axes = containing_flow_axes.physical_size(logical_requests);
        // The old Native NoParent fixture explicitly supplied these percentage
        // bases. It supplied no independent ICB dimensions or ancestor scrollport.
        let percentage_parent = requested_axes.map(|axis| axis.space().definite_size());
        let fallback = crate::OrthogonalFallbackOf::new(
            crate::Size::NONE,
            crate::NearestScrollportOf::Absent,
            crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
        );
        crate::LayoutRootRequestOf::standalone(
            crate::SizingRequestOf::new(axes, fallback),
            containing_flow_axes,
            percentage_parent,
        )
    }
}
fn stage(session: &mut ComputeSession<'_, PublicLayoutTreeOf<f64>>) {
    compute_root(session, 0, request()).unwrap();
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
                Some(text::source::Id::from_u64(1578081)),
                71,
                text::source::Range::new(0, 1),
                text::source::Range::new(0, 1)
            ),
            (
                Some(text::source::Id::from_u64(1578082)),
                71,
                text::source::Range::new(0, 3),
                text::source::Range::new(1, 4)
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
            (text::source::Range::new(0, 2), 11.0, 13.0),
            (text::source::Range::new(2, 4), 24.0, 13.0)
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
            (text::source::Range::new(0, 1), 58, 10.0, 11.0),
            (text::source::Range::new(1, 2), 58, 10.0, 8.0),
            (text::source::Range::new(2, 3), 58, 10.0, 21.0),
            (text::source::Range::new(3, 4), 58, 10.0, 21.0)
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
                Some(text::allocation::InlineBoundaryAllocation::CONTRIBUTION_ZERO)
            );
            assert_eq!(
                allocation.strut,
                Some(text::allocation::ControlStrut {
                    baseline: 0.0,
                    line_extent: 0.0,
                    alignment: text::allocation::LineBoxAlignment::Baseline
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
    let mut session = ComputeSession::with_flow_unobserved(&flow, vec![]);
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
    let mut producer = ComputeSession::with_flow_unobserved(&flow, vec![]);
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
    // A fresh real same-flow session starts a work ledger at0. Its
    // actual private global bridge consumes the real native pass products.
    let session = ComputeSession::with_flow_unobserved(&flow, vec![]);
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
            let LayoutErrorKindOf::ResourceLimit(error) = error.kind() else {
                panic!("global scope work must be a layout resource limit")
            };
            assert_eq!(error.code, text::ErrorCode::ScopeAlignmentRefused);
            let Some(text::ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
                panic!("missing actual global scope resource detail")
            };
            let text::scope_alignment::ScopeAlignmentError::Limit {
                kind,
                required,
                limit,
            } = *reason
            else {
                panic!("global refusal must not become a feedback cycle or miss")
            };
            assert_eq!(
                kind,
                text::scope_alignment::ScopeAlignmentLimitKind::StructuralWork
            );
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
fn source_global_pass_bridge_admits_scope_payloads_and_propagates_typed_work_refusal() {
    global_bridge(WORK);
    global_bridge(WORK + PASS_WORK);
}

fn limits() -> text::scope_alignment::ScopeAlignmentLimits {
    text::scope_alignment::ScopeAlignmentLimits::new(10000, 100, WORK, 16 << 20)
}

/// A session whose entry-source custody is installed, so child staging is
/// funded from the captured scope-alignment owner.
fn funded_session<'a>(
    flow: &'a LayoutFlowInput<'a, PublicLayoutTreeOf<f64>>,
) -> ComputeSession<'a, PublicLayoutTreeOf<f64>> {
    let mut session = ComputeSession::with_flow_unobserved(flow, vec![]);
    session.entry_sources = Some(Arc::new(std::sync::Mutex::new(
        crate::engine::entry_sources::SourceMemo::new(None),
    )));
    session
}

fn assert_staging_failure_is_resource_limit(
    error: &LayoutErrorOf<u32, f64>,
    limit: text::scope_alignment::ScopeAlignmentLimitKind,
) {
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0), "{error:?}");
    assert_eq!(error.operation(), LayoutOperation::ChildLayout, "{error:?}");
    let LayoutErrorKindOf::ResourceLimit(refusal) = error.kind() else {
        panic!(
            "a spent {limit:?} budget during child staging is a resource limit of layout, \
             not a Text protocol refusal: {error:?}"
        )
    };
    let Some(text::ErrorDetail::ScopeAlignment {
        reason: text::scope_alignment::ScopeAlignmentError::Limit { kind, .. },
    }) = refusal.detail()
    else {
        panic!("the resource limit must keep the exhausted budget: {refusal:?}")
    };
    assert_eq!(*kind, limit);
    let source = std::error::Error::source(error).expect("a resource limit names its cause");
    assert!(source.downcast_ref::<text::Error>().is_some());
    assert!(
        error
            .to_string()
            .starts_with("child layout failed at node 0:"),
        "{error}"
    );
}

#[test]
fn child_staging_with_spent_scope_work_reports_a_resource_limit_not_text() {
    let fixture = Fixture::new(WORK);
    let flow = fixture.flow();

    // Control: the same funded session stages a child while work remains.
    let control = funded_session(&flow);
    control
        .child_staging(0, super::FloatChildTrial::InFlow, None)
        .unwrap();

    let mut spent = funded_session(&flow);
    let resources = spent.text_computation().unwrap().scope_resources.clone();
    resources.visit(limits(), WORK).unwrap();
    let error = spent
        .child_staging(0, super::FloatChildTrial::InFlow, None)
        .err()
        .expect("staging must refuse once the structural work budget is spent");
    assert_staging_failure_is_resource_limit(
        &error,
        text::scope_alignment::ScopeAlignmentLimitKind::StructuralWork,
    );
}

#[test]
fn child_staging_with_spent_scope_records_reports_a_resource_limit_not_text() {
    let fixture = Fixture::new(WORK);
    let flow = fixture.flow();
    let mut spent = funded_session(&flow);
    let resources = spent.text_computation().unwrap().scope_resources.clone();
    let mut admitted = 0_usize;
    loop {
        match resources.materialize(limits(), 0, 1) {
            Ok(()) => admitted += 1,
            Err(_) => break,
        }
        assert!(admitted <= 10_000, "the record ceiling must be reachable");
    }
    let error = spent
        .child_staging(0, super::FloatChildTrial::InFlow, None)
        .err()
        .expect("staging must refuse once the record budget is spent");
    assert_staging_failure_is_resource_limit(
        &error,
        text::scope_alignment::ScopeAlignmentLimitKind::Records,
    );
}

fn cache_input() -> ComputeInputOf<f64> {
    ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(20.0), None),
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::Normal,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}

fn cache_output() -> ComputeOutputOf<f64> {
    ComputeOutputOf::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0))
}

/// A staged trial that owns funded cache bookkeeping, with its shared scope
/// resources.
fn funded_trial<'a>(
    session: &ComputeSession<'a, PublicLayoutTreeOf<f64>>,
) -> (
    ComputeSession<'a, PublicLayoutTreeOf<f64>>,
    crate::text_layout::scope_alignment::ScopeAlignmentResources<f64>,
) {
    let mut trial = session
        .child_staging(0, super::FloatChildTrial::InFlow, None)
        .unwrap();
    assert!(trial.entry_storage.is_some(), "the trial is funded");
    let resources = trial.text_computation().unwrap().scope_resources.clone();
    (trial, resources)
}

/// Spends the structural work budget until exactly `keep` units remain. A
/// refused admission charges nothing, so halving the step reaches the ceiling.
fn spend_work_leaving(
    resources: &crate::text_layout::scope_alignment::ScopeAlignmentResources<f64>,
    keep: usize,
) {
    let lowered =
        text::scope_alignment::ScopeAlignmentLimits::new(10000, 100, WORK - keep, 16 << 20);
    let mut step = WORK;
    while step > 0 {
        if resources.visit(lowered, step).is_err() {
            step /= 2;
        }
    }
}

fn exhaust_records(resources: &crate::text_layout::scope_alignment::ScopeAlignmentResources<f64>) {
    for _ in 0..=10_000 {
        if resources.materialize(limits(), 0, 1).is_err() {
            return;
        }
    }
    panic!("the record ceiling must be reachable");
}

fn assert_cache_failure_is_resource_limit(
    error: &LayoutErrorOf<u32, f64>,
    limit: text::scope_alignment::ScopeAlignmentLimitKind,
) {
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0), "{error:?}");
    assert_eq!(error.operation(), LayoutOperation::ChildLayout, "{error:?}");
    let LayoutErrorKindOf::ResourceLimit(refusal) = error.kind() else {
        panic!(
            "a spent {limit:?} budget during cache bookkeeping is a resource limit of \
             layout, not a Text protocol refusal: {error:?}"
        )
    };
    assert!(
        matches!(refusal.detail(), Some(text::ErrorDetail::ScopeAlignment {
            reason: text::scope_alignment::ScopeAlignmentError::Limit { kind, .. },
        }) if *kind == limit),
        "{refusal:?}"
    );
}

#[test]
fn cache_store_with_spent_scope_records_reports_a_resource_limit_not_text() {
    let fixture = Fixture::new(WORK);
    let flow = fixture.flow();
    let session = funded_session(&flow);
    let (mut trial, resources) = funded_trial(&session);
    exhaust_records(&resources);
    let error = trial
        .cache_store(
            0,
            &cache_input(),
            crate::CacheKeyContext::new(),
            cache_output(),
        )
        .expect_err("a store must refuse once the record budget is spent");
    assert_cache_failure_is_resource_limit(
        &error,
        text::scope_alignment::ScopeAlignmentLimitKind::Records,
    );
}

#[test]
fn cache_store_replacing_a_stored_entry_with_spent_scope_work_reports_a_resource_limit_not_text() {
    let fixture = Fixture::new(WORK);
    let flow = fixture.flow();
    let mut refused = 0;
    // The funded store charges a small, fixed amount of work before it scans
    // the stored entries. Every remaining budget up to that amount must report
    // a resource limit wherever the budget runs out.
    for keep in 0..64 {
        // Each iteration owns its resources: spent work is never refunded.
        let session = funded_session(&flow);
        let (mut trial, resources) = funded_trial(&session);
        // Control: the first store is funded while work remains.
        trial
            .cache_store(
                0,
                &cache_input(),
                crate::CacheKeyContext::new(),
                cache_output(),
            )
            .unwrap();
        spend_work_leaving(&resources, keep);
        match trial.cache_store(
            0,
            &cache_input(),
            crate::CacheKeyContext::new(),
            cache_output(),
        ) {
            Ok(()) => {}
            Err(error) => {
                refused += 1;
                assert_cache_failure_is_resource_limit(
                    &error,
                    text::scope_alignment::ScopeAlignmentLimitKind::StructuralWork,
                );
            }
        }
    }
    assert!(refused > 0, "a spent work budget must refuse the store");
}

#[test]
fn cache_clear_with_spent_scope_records_reports_a_resource_limit_not_text() {
    let fixture = Fixture::new(WORK);
    let flow = fixture.flow();
    let session = funded_session(&flow);
    let (mut trial, resources) = funded_trial(&session);
    exhaust_records(&resources);
    let error = trial
        .cache_clear(0)
        .expect_err("a clear must refuse once the record budget is spent");
    assert_cache_failure_is_resource_limit(
        &error,
        text::scope_alignment::ScopeAlignmentLimitKind::Records,
    );
}

#[cfg(test)]
#[path = "percentage_basis_phase_refusal_tests.rs"]
mod percentage_basis_phase_refusal_tests;
