//! Proposed child module of engine/session.rs, using its real private session.
//! This is an owning test, not a public parent-route/ComputeSize coverage claim.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use surgeist_text as t;

fn fixture<S: LayoutScalar>(
    force: bool,
) -> (
    PublicLayoutTreeOf<S>,
    t::prepared::PreparedText,
    WrappingInput<u32>,
) {
    use t::capture::SourceProcessingEvent as E;
    let inline = || t::capture::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: None,
    };
    let captured = std::sync::Arc::new(
        t::capture::process_contextual_source(t::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: t::source::Source::identified("", t::source::Id::from_u64(658300), 51),
            segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::capture::SourceProcessingEnvelope::Complete,
            origins: vec![],
            controls: vec![],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: if force {
                vec![t::capture::ComputedLineBoundaryInput::Scope {
                    scope: 2,
                    side: t::capture::SourceBoxSide::Before,
                }]
            } else {
                vec![]
            },
            scopes: vec![
                t::capture::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: t::capture::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: t::style::Style {
                        wrap: t::style::Wrap::None,
                        hyphens: t::hyphenation::Hyphens::None,
                        ..crate::test_support::text::ahem_style(10.0, 10.0)
                    },
                    used_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
                },
                inline(),
                inline(),
            ],
            paragraphs: vec![t::capture::SourceProcessingParagraph {
                base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                E::EnterScope(2),
                E::ExitScope(2),
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let scopes = [
        captured.captured_scope(1).unwrap(),
        captured.captured_scope(2).unwrap(),
    ];
    let analyzed = t::analyzed::AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        t::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::analyzed::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: t::style::Wrap::None,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut fonts = t::System::default();
    fonts
        .register_fonts(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            .to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    let prepared = fonts
        .prepare(&analyzed, t::prepared::PreparationOptions::default())
        .unwrap();
    assert!(prepared.origin_views().next().is_none());
    assert!(prepared.control_views().next().is_none());
    assert!(prepared.atomic_views().next().is_none());
    let tree = PublicLayoutTreeOf::<S>::new().children(0, []).style(
        0,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.0)),
                PreferredSizeOf::px(S::from_f64(50.0)),
            ),
            ..NodeInputOf::default()
        },
    );
    let s = GeneratedBoxRef::Inline(InlineBoxRef::Scope(scopes[0].clone()));
    let inner = GeneratedBoxRef::Inline(InlineBoxRef::Scope(scopes[1].clone()));
    let child = |box_| GeneratedChildBoxInput {
        box_,
        participation: GeneratedBoxParticipation::InFlow,
    };
    let row = |parent, children| InlineChildOrderInput {
        parent,
        children,
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    };
    let inventory = WrappingInput {
        native_inline_boxes: Vec::new(),
        gaps: Vec::new(),
        scopes: scopes
            .iter()
            .zip([(16.0, 20.0), (12.0, 30.0)])
            .map(
                |(scope, (baseline, line_extent))| WrappingScopeInput::Captured {
                    scope: scope.clone(),
                    wrap_inside: WrapInside::Auto,
                    edges: WrapEdges::AUTO,
                    metric_contribution: Some(InlineBoxMetricContribution::Strut(
                        t::allocation::LineBoxStrut {
                            scope: None,
                            metrics: t::allocation::LineStrut {
                                baseline,
                                line_extent,
                            },
                            alignment: t::allocation::LineBoxAlignment::Baseline,
                        },
                    )),
                },
            )
            .collect(),
        child_order: vec![
            row(GeneratedBoxRef::Node(0), vec![child(s.clone())]),
            row(s, vec![child(inner.clone())]),
            row(inner, vec![]),
        ],
    };
    (tree, prepared, inventory)
}

fn known_size_baseline<S: LayoutScalar>() {
    for force in [false, true] {
        let (tree, prepared, inventory) = fixture::<S>(force);
        let expected_scope = match &inventory.scopes[0] {
            WrappingScopeInput::Captured { scope, .. } => scope.clone(),
            _ => unreachable!("fixture declares its actual captured outer box"),
        };
        let binding = TextParagraphBinding {
            inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
            prepared: prepared.clone(),
            container: 0,
            nodes: vec![],
            origins: vec![],
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        };
        let flow =
            LayoutFlowInput::try_new(&tree, 0, vec![binding], inventory, Vec::new()).unwrap();
        let known = Size::new(Some(S::from_f64(30.0)), Some(S::from_f64(50.0)));
        let parent = Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(100.0)));
        let containing = ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        );
        let available = Size::splat(crate::AxisSizingRequestOf::new(
            crate::AvailableSpaceOf::Definite(
                crate::NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
            ),
            crate::SizingOperation::Normal,
        ));
        // The actual public constructor issues ComputeSize/ContentSize/Both;
        // do not substitute a viewport root or a PerformLayout parent probe.
        let input = ComputeInputOf::leaf_content_size(
            known,
            parent,
            containing,
            crate::SizingRequestOf::new(
                available,
                crate::OrthogonalFallbackOf::new(
                    crate::Size::NONE,
                    crate::NearestScrollportOf::Unknown,
                    crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        )
        .unwrap();
        let mut session = ComputeSession::with_flow_unobserved(&flow, flow.invalidated.clone());
        let output = session.compute_child(0, input).unwrap();
        assert_eq!(output.size, Size::new(S::from_f64(30.0), S::from_f64(50.0)));
        assert_eq!(output.first_baselines.y, force.then_some(S::from_f64(16.0)));
        assert_eq!(output.last_baselines.y, force.then_some(S::from_f64(16.0)));

        // Size queries need not publish pending placements. A fresh
        // PerformLayout counterpart exposes the same semantic source/box receipt
        // through the existing owning session, without adding a test-only API.
        let mut placed = ComputeSession::with_flow_unobserved(&flow, flow.invalidated.clone());
        let output = placed
            .compute_child(
                0,
                ComputeInputOf::leaf_layout(
                    known,
                    parent,
                    containing,
                    crate::SizingRequestOf::new(
                        available,
                        crate::OrthogonalFallbackOf::new(
                            crate::Size::NONE,
                            crate::NearestScrollportOf::Unknown,
                            crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                        ),
                    ),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(output.size, Size::new(S::from_f64(30.0), S::from_f64(50.0)));
        assert_eq!(output.first_baselines.y, force.then_some(S::from_f64(16.0)));
        let pending = &placed.text.as_ref().unwrap().pending;
        if !force {
            // Positive scope struts without a Force remain AssociationOnly.
            assert!(pending.is_empty());
            continue;
        }
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].prepared, prepared);
        assert!(pending[0].nodes.is_empty());
        let [
            t::placement::PlacementEntry::Formatted(first),
            t::placement::PlacementEntry::NonFormatted(end),
        ] = pending[0].entries.as_slice()
        else {
            panic!("actual Forced placement and nonformatted Paragraph completion are retained");
        };
        let offers = prepared
            .opportunity_views()
            .map(|o| o.handle())
            .collect::<Vec<_>>();
        let first_selected = first.line.selected_line();
        assert_eq!(
            first_selected.request().context.end,
            t::selection::LineEnd::Forced
        );
        assert_eq!(
            first_selected.request().termination,
            t::selection::PreparedTermination::Ordinary(offers[0].clone())
        );
        assert_eq!(first.line.feedback().baseline, 16.0);
        assert_eq!(first.line.feedback().line_height, 20.0);
        assert_eq!(first.anchor.baseline, 16.0);
        assert_eq!(
            first_selected.request().line_box_struts,
            [t::allocation::LineBoxStrut {
                scope: Some(expected_scope),
                metrics: t::allocation::LineStrut {
                    baseline: 16.0,
                    line_extent: 20.0
                },
                alignment: t::allocation::LineBoxAlignment::Baseline
            }]
        );
        let final_line = end.line.finalized_line();
        let final_selected = final_line.selected_line();
        let request = final_selected.request();
        assert_eq!(final_line.prepared_text(), prepared);
        assert_eq!(request.context.end, t::selection::LineEnd::Paragraph);
        assert_eq!(
            request.termination,
            t::selection::PreparedTermination::Ordinary(offers[1].clone())
        );
        assert_eq!(request.start, first_selected.request().end);
        assert_eq!(request.end, offers[1].view().boundary());
        assert!(request.participants.is_empty());
        assert_eq!(request.start.view().source_before().get(), 0);
        assert_eq!(request.end.view().source_after().get(), 0);
        assert_eq!(final_line.feedback().baseline, 16.0);
        assert_eq!(final_line.feedback().line_height, 34.0);
        assert_eq!(final_line.feedback().advance, 0.0);
        assert_eq!(end.origin.y, 20.0);
        // First/max-before16, max-after18:34 remains nominal only. No
        // baseline50 may be synthesized from this fixed-height enclosing box.
    }
}

#[test]
fn known_size_node_free_semantic_block_keeps_actual_forced_baseline_in_size_query() {
    known_size_baseline::<f32>();
    known_size_baseline::<f64>();
}
